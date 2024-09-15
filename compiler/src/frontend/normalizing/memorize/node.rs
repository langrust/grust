prelude! {
    petgraph::graphmap::GraphMap,
    hir::{ IdentifierCreator, Memory, Component, ComponentDefinition },
}

impl Component {
    /// Create memory for HIR node's unitary nodes.
    ///
    /// Store buffer for followed by expressions and unitary node applications.
    /// Transform followed by expressions in signal call.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     x_1: int = 0 fby v;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     x_1: int = mem;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// memory test {
    ///     buffers: {
    ///         mem: int = 0 fby v;
    ///     },
    ///     called_nodes: {
    ///         memmy_node_o_: (my_node, o);
    ///     },
    /// }
    /// ```
    pub fn memorize(&mut self, symbol_table: &mut SymbolTable) {
        match self {
            Component::Definition(comp_def) => comp_def.memorize(symbol_table),
            Component::Import(_) => (),
        }
    }
}

impl ComponentDefinition {
    /// Create memory for HIR node's unitary nodes.
    ///
    /// Store buffer for followed by expressions and unitary node applications.
    /// Transform followed by expressions in signal call.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     x_1: int = 0 fby v;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     x_1: int = mem;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// memory test {
    ///     buffers: {
    ///         mem: int = 0 fby v;
    ///     },
    ///     called_nodes: {
    ///         memmy_node_o_: (my_node, o);
    ///     },
    /// }
    /// ```
    pub fn memorize(&mut self, symbol_table: &mut SymbolTable) {
        // create an IdentifierCreator, a local SymbolTable and Memory
        let mut identifier_creator = IdentifierCreator::from(self.get_signals_names(symbol_table));
        symbol_table.local();
        let mut memory = Memory::new();

        self.statements.iter_mut().for_each(|statement| {
            statement.memorize(
                &mut identifier_creator,
                &mut memory,
                &mut self.contract,
                symbol_table,
            )
        });

        // drop IdentifierCreator (auto), local SymbolTable and set Memory
        symbol_table.global();
        self.memory = memory;

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
