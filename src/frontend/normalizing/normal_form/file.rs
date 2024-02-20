use std::collections::BTreeMap;

use crate::{hir::file::File, symbol_table::SymbolTable};

impl File {
    /// Change HIR file into a normal form.
    ///
    /// The normal form of a node is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
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
        let mut nodes_reduced_graphs = BTreeMap::new();
        // get every nodes' graphs
        self.nodes.iter().for_each(|node| {
            node.unitary_nodes.values().for_each(|unitary_node| {
                debug_assert!(nodes_reduced_graphs
                    .insert(unitary_node.id.clone(), unitary_node.graph.clone())
                    .is_none())
            })
        });
        // get optional component's graph
        if let Some(component) = self.component.as_ref() {
            component.unitary_nodes.values().for_each(|unitary_node| {
                debug_assert!(nodes_reduced_graphs
                    .insert(unitary_node.id.clone(), unitary_node.graph.clone())
                    .is_none())
            })
        };

        self.nodes
            .iter_mut()
            .for_each(|node| node.normal_form(&nodes_reduced_graphs, symbol_table));
        if let Some(component) = self.component.as_mut() {
            component.normal_form(&nodes_reduced_graphs, symbol_table)
        }

        // Debug: test it is in normal form
        debug_assert!(self.is_normal_form());
    }
}
