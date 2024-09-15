use petgraph::graphmap::GraphMap;

prelude! {
    hir::{ IdentifierCreator, Memory, Component, ComponentDefinition, stmt, Stmt, stream },
}

use super::Union;

impl Component {
    /// Inline node application when it is needed.
    ///
    /// Inlining needed for "shifted causality loop".
    ///
    /// # Example:
    /// ```GR
    /// node semi_fib(i: int) {
    ///     out o: int = 0 fby (i + 1 fby i);
    /// }
    /// node fib_call() {
    ///    out fib: int = semi_fib(fib).o;
    /// }
    /// ```
    /// In this example, `fib_call` calls `semi_fib` with the same input and output signal. There is
    /// no causality loop, `o` depends on the memory of `i`.
    ///
    /// We need to inline the code, the output `fib` is defined before the input `fib`, which can
    /// not be computed by a function call.
    pub fn inline_when_needed(
        &mut self,
        unitary_nodes: &HashMap<usize, Component>,
        symbol_table: &mut SymbolTable,
    ) {
        match self {
            Component::Definition(comp_def) => {
                comp_def.inline_when_needed(unitary_nodes, symbol_table)
            }
            Component::Import(_) => (),
        }
    }

    /// Instantiate unitary node's statements with inputs.
    ///
    /// It will return new statements where the input signals are instantiated by expressions. New
    /// statements should have fresh id according to the calling node.
    ///
    /// # Example
    ///
    /// ```GR
    /// node to_be_inlined(i: int) {
    ///     o: int = 0 fby j;
    ///     out j: int = i + 1;
    /// }
    ///
    /// node calling_node(i: int) {
    ///     out o: int = to_be_inlined(o);
    ///     j: int = i * o;
    /// }
    /// ```
    ///
    /// The call to `to_be_inlined` will generate th following statements:
    ///
    /// ```GR
    /// o: int = 0 fby j_1;
    /// j_1: int = o + 1;
    /// ```
    pub fn instantiate_statements_and_memory(
        &self,
        identifier_creator: &mut IdentifierCreator,
        inputs: &[(usize, stream::Expr)],
        new_output_pattern: Option<stmt::Pattern>,
        symbol_table: &mut SymbolTable,
    ) -> (Vec<Stmt<stream::Expr>>, Memory) {
        match self {
            Component::Definition(comp_def) => comp_def.instantiate_statements_and_memory(
                identifier_creator,
                inputs,
                new_output_pattern,
                symbol_table,
            ),
            Component::Import(_) => (vec![], Memory::new()),
        }
    }
}

impl ComponentDefinition {
    /// Inline node application when it is needed.
    ///
    /// Inlining needed for "shifted causality loop".
    ///
    /// # Example:
    /// ```GR
    /// node semi_fib(i: int) {
    ///     out o: int = 0 fby (i + 1 fby i);
    /// }
    /// node fib_call() {
    ///    out fib: int = semi_fib(fib).o;
    /// }
    /// ```
    /// In this example, `fib_call` calls `semi_fib` with the same input and output signal. There is
    /// no causality loop, `o` depends on the memory of `i`.
    ///
    /// We need to inline the code, the output `fib` is defined before the input `fib`, which can
    /// not be computed by a function call.
    pub fn inline_when_needed(
        &mut self,
        unitary_nodes: &HashMap<usize, Component>,
        symbol_table: &mut SymbolTable,
    ) {
        // create identifier creator containing the signals
        let mut identifier_creator = IdentifierCreator::from(self.get_signals_names(symbol_table));

        // compute new statements for the unitary node
        let mut new_statements: Vec<Stmt<stream::Expr>> = vec![];
        std::mem::take(&mut self.statements)
            .into_iter()
            .for_each(|statement| {
                let mut retrieved_statements = statement.inline_when_needed_recursive(
                    &mut self.memory,
                    &mut identifier_creator,
                    symbol_table,
                    unitary_nodes,
                );
                new_statements.append(&mut retrieved_statements)
            });

        // update node's unitary node
        self.update_statements(&new_statements)
    }

    /// Instantiate unitary node's statements with inputs.
    ///
    /// It will return new statements where the input signals are instantiated by expressions. New
    /// statements should have fresh id according to the calling node.
    ///
    /// # Example
    ///
    /// ```GR
    /// node to_be_inlined(i: int) {
    ///     o: int = 0 fby j;
    ///     out j: int = i + 1;
    /// }
    ///
    /// node calling_node(i: int) {
    ///     out o: int = to_be_inlined(o);
    ///     j: int = i * o;
    /// }
    /// ```
    ///
    /// The call to `to_be_inlined` will generate th following statements:
    ///
    /// ```GR
    /// o: int = 0 fby j_1;
    /// j_1: int = o + 1;
    /// ```
    pub fn instantiate_statements_and_memory(
        &self,
        identifier_creator: &mut IdentifierCreator,
        inputs: &[(usize, stream::Expr)],
        new_output_pattern: Option<stmt::Pattern>,
        symbol_table: &mut SymbolTable,
    ) -> (Vec<Stmt<stream::Expr>>, Memory) {
        // create the context with the given inputs
        let mut context_map = inputs
            .iter()
            .map(|(input, expression)| (*input, Union::I2(expression.clone())))
            .collect::<HashMap<_, _>>();

        // add output signals to context
        new_output_pattern.map(|pattern| {
            let signals = pattern.identifiers();
            symbol_table
                .get_node_outputs(self.id)
                .iter()
                .zip(signals)
                .for_each(|((_, output_id), new_output_id)| {
                    context_map.insert(*output_id, Union::I1(new_output_id));
                })
        });

        // add identifiers of the inlined statements to the context
        self.statements.iter().for_each(|statement| {
            statement.add_necessary_renaming(identifier_creator, &mut context_map, symbol_table)
        });
        // add identifiers of the inlined memory to the context
        self.memory
            .add_necessary_renaming(identifier_creator, &mut context_map, symbol_table);

        // reduce statements according to the context
        let statements = self
            .statements
            .iter()
            .map(|statement| statement.replace_by_context(&context_map))
            .collect();

        // reduce memory according to the context
        let memory = self.memory.replace_by_context(&context_map, &symbol_table);

        (statements, memory)
    }

    /// Update unitary node statements and add the corresponding dependency graph.
    fn update_statements(&mut self, new_statements: &[Stmt<stream::Expr>]) {
        // put new statements in unitary node
        self.statements = new_statements.to_vec();
        // add a dependency graph to the unitary node
        let mut graph = GraphMap::new();
        self.get_signals_id().iter().for_each(|signal_id| {
            graph.add_node(*signal_id);
        });
        self.statements
            .iter()
            .for_each(|statement| statement.add_to_graph(&mut graph));
        self.graph = graph;
    }
}
