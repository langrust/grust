prelude! {
    petgraph::graphmap::GraphMap,
    graph::*,
    hir::{IdentifierCreator, Component, ComponentDefinition},
}

impl Component {
    /// Change HIR node into a normal form.
    ///
    /// The normal form of a node is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(s: int, v: int, g: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    ///     out y: int = other_node(g-1, v).o;
    /// }
    /// ```
    ///
    /// The above node contains the following unitary nodes:
    ///
    /// ```GR
    /// node test_x(s: int, v: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    /// }
    /// node test_y(v: int, g: int) {
    ///     out y: int = other_node(g-1, v).o;
    /// }
    /// ```
    ///
    /// Which are transformed into:
    ///
    /// ```GR
    /// node test_x(s: int, v: int) {
    ///     x_1: int = v*2;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// node test_y(v: int, g: int) {
    ///     x: int = g-1;
    ///     out y: int = other_node(x_1, v).o;
    /// }
    /// ```
    pub fn normal_form(
        &mut self,
        nodes_reduced_graphs: &HashMap<usize, DiGraphMap<usize, Label>>,
        symbol_table: &mut SymbolTable,
    ) {
        match self {
            Component::Definition(comp_def) => {
                comp_def.normal_form(nodes_reduced_graphs, symbol_table)
            }
            Component::Import(_) => (),
        }
    }
}

impl ComponentDefinition {
    /// Change HIR node into a normal form.
    ///
    /// The normal form of a node is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(s: int, v: int, g: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    ///     out y: int = other_node(g-1, v).o;
    /// }
    /// ```
    ///
    /// The above node contains the following unitary nodes:
    ///
    /// ```GR
    /// node test_x(s: int, v: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    /// }
    /// node test_y(v: int, g: int) {
    ///     out y: int = other_node(g-1, v).o;
    /// }
    /// ```
    ///
    /// Which are transformed into:
    ///
    /// ```GR
    /// node test_x(s: int, v: int) {
    ///     x_1: int = v*2;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// node test_y(v: int, g: int) {
    ///     x: int = g-1;
    ///     out y: int = other_node(x_1, v).o;
    /// }
    /// ```
    pub fn normal_form(
        &mut self,
        nodes_reduced_graphs: &HashMap<usize, DiGraphMap<usize, Label>>,
        symbol_table: &mut SymbolTable,
    ) {
        // create an IdentifierCreator and a local SymbolTable
        let mut identifier_creator = IdentifierCreator::from(self.get_signals_names(symbol_table));
        symbol_table.local();

        let ComponentDefinition { statements, .. } = self;

        *statements = statements
            .clone()
            .into_iter()
            .flat_map(|equation| {
                equation.normal_form(nodes_reduced_graphs, &mut identifier_creator, symbol_table)
            })
            .collect();

        // drop IdentifierCreator (auto) and local SymbolTable
        symbol_table.global();

        // add a dependency graph to the node
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
