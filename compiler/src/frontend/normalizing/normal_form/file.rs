prelude! {
    hir::File,
}

impl File {
    /// Change HIR file into a normal form.
    ///
    /// The normal form of a node is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    /// - no rising edges (replaced by |test| test && ! (false fby test))
    ///
    /// The normal form of a flow expression is as follows:
    /// - flow expressions others than identifiers are root expression
    /// - then, arguments are only identifiers
    ///
    /// # Example
    ///
    /// ```GR
    /// function test(i: int) -> int {
    ///     let x: int = i;
    ///     return x;
    /// }
    /// node my_node(x: int, y: int) {
    ///     out o: int = x*y;
    /// }
    /// node other_node(x: int, y: int) {
    ///     out o: int = x*y;
    /// }
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
    ///
    /// This example is tested in source.
    pub fn normal_form(&mut self, symbol_table: &mut SymbolTable) {
        let mut nodes_reduced_graphs = HashMap::new();
        // get every nodes' graphs
        self.components.iter().for_each(|node| {
            let test_first_insert =
                nodes_reduced_graphs.insert(node.get_id().clone(), node.get_graph().clone());
            debug_assert!(test_first_insert.is_none())
        });
        // normalize nodes
        self.components
            .iter_mut()
            .for_each(|node| node.normal_form(&nodes_reduced_graphs, symbol_table));

        // normalize interface
        self.interface.normal_form(symbol_table);

        // Debug: test it is in normal form
        debug_assert!(self.is_normal_form());
    }
}
