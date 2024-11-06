prelude! {
    petgraph::algo::toposort,
    graph::*,
    hir::{ IdentifierCreator, Memory, Component, stream },
}

use super::Union;

impl stream::Stmt {
    /// Add the statement identifier to the identifier creator.
    ///
    /// It will add the statement identifier to the identifier creator. If the identifier already
    /// exists, then the new identifier created by the identifier creator will be added to the
    /// renaming context.
    pub fn add_necessary_renaming(
        &self,
        identifier_creator: &mut IdentifierCreator,
        context_map: &mut HashMap<usize, Union<usize, stream::Expr>>,
        symbol_table: &mut SymbolTable,
    ) {
        // create fresh identifiers for the new statement
        let local_signals = self.pattern.identifiers();
        for signal_id in local_signals {
            let name = symbol_table.get_name(signal_id);
            let scope = symbol_table.get_scope(signal_id).clone();
            let fresh_name = identifier_creator.new_identifier(name);
            if Scope::Output != scope && &fresh_name != name {
                let typing = Some(symbol_table.get_type(signal_id).clone());
                let fresh_id = symbol_table.insert_fresh_signal(fresh_name, scope, typing);
                let _unique = context_map.insert(signal_id, Union::I1(fresh_id));
                debug_assert!(_unique.is_none());
            }
        }
    }

    /// Replace identifier occurrence by element in context.
    ///
    /// It will return a new statement where the expression has been modified according to the
    /// context:
    ///
    /// - if an identifier is mapped to another identifier, then rename all occurrence of the
    ///   identifier by the new one
    /// - if the identifier is mapped to an expression, then replace all call to the identifier by
    ///   the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2, z -> c]`, a call to the function with the statement `z =
    /// x + y` will return `c = a + b/2`.
    pub fn replace_by_context(
        &self,
        context_map: &HashMap<usize, Union<usize, stream::Expr>>,
    ) -> stream::Stmt {
        let mut new_statement = self.clone();

        // replace statement's identifiers by the new ones
        let local_signals = new_statement.pattern.identifiers_mut();
        for signal_id in local_signals {
            if let Some(element) = context_map.get(&signal_id) {
                match element {
                    Union::I1(new_id)
                    | Union::I2(stream::Expr {
                        kind:
                            stream::Kind::Expression {
                                expression: hir::expr::Kind::Identifier { id: new_id },
                            },
                        ..
                    }) => {
                        *signal_id = new_id.clone();
                    }
                    Union::I2(_) => unreachable!(),
                }
            }
        }

        // replace identifiers in statement's expression
        new_statement.expression.replace_by_context(context_map);

        new_statement
    }

    /// Inline node application when it is needed.
    ///
    /// Inlining needed for "shifted causality loop".
    ///
    /// # Example:
    /// ```GR
    /// node semi_fib(i: int) {
    ///     out o: int = 0 fby (i + 1 fby i);
    /// }
    /// ```
    /// In this example, an statement `fib: int = semi_fib(fib).o` calls
    /// `semi_fib` with the same input and output signal.
    /// There is no causality loop, `o` depends on the memory of `i`.
    ///
    /// We need to inline the code, the output `fib` is defined before the input `fib`,
    /// which can not be computed by a function call.
    pub fn inline_when_needed_recursive(
        self,
        memory: &mut Memory,
        identifier_creator: &mut IdentifierCreator,
        symbol_table: &mut SymbolTable,
        nodes: &HashMap<usize, Component>,
    ) -> Vec<stream::Stmt> {
        let mut current_statements = vec![self.clone()];
        let mut new_statements =
            self.inline_when_needed(memory, identifier_creator, symbol_table, nodes);
        while current_statements != new_statements {
            current_statements = new_statements;
            new_statements = current_statements
                .clone()
                .into_iter()
                .flat_map(|statement| {
                    statement.inline_when_needed(memory, identifier_creator, symbol_table, nodes)
                })
                .collect();
        }
        new_statements
    }

    fn inline_when_needed(
        self,
        memory: &mut Memory,
        identifier_creator: &mut IdentifierCreator,
        symbol_table: &mut SymbolTable,
        nodes: &HashMap<usize, Component>,
    ) -> Vec<stream::Stmt> {
        match &self.expression.kind {
            stream::Kind::NodeApplication {
                called_node_id,
                inputs,
                memory_id,
                ..
            } => {
                // a loop in the graph induces that "node call" inputs depends on output
                let is_loop = {
                    let mut graph = DiGraphMap::new();
                    let outs = self.pattern.identifiers();
                    let in_deps = inputs.iter().flat_map(|(_, expr)| expr.get_dependencies());
                    for (to, label) in in_deps {
                        for from in outs.iter() {
                            graph.add_edge(*from, *to, label.clone());
                        }
                    }
                    toposort(&graph, None).is_err()
                };

                // then node call must be inlined
                if is_loop {
                    let called_unitary_node = nodes.get(&called_node_id).unwrap();

                    // get statements from called node, with corresponding inputs
                    let (retrieved_statements, retrieved_memory) = called_unitary_node
                        .instantiate_statements_and_memory(
                            identifier_creator,
                            inputs,
                            Some(self.pattern),
                            symbol_table,
                        );

                    // remove called node from memory
                    memory.remove_called_node(memory_id.unwrap());

                    memory.combine(retrieved_memory);
                    retrieved_statements
                } else {
                    // otherwise, just return self
                    vec![self]
                }
            }
            _ => vec![self],
        }
    }
}
