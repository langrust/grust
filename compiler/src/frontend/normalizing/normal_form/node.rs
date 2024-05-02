use std::collections::HashMap;

use itertools::Itertools;
use petgraph::graphmap::DiGraphMap;

use crate::{common::label::Label, hir::node::Node, symbol_table::SymbolTable};

impl Node {
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
        self.unitary_nodes
            .values_mut()
            .sorted_by_key(|unitary_node| unitary_node.id)
            .for_each(|unitary_node| unitary_node.normal_form(nodes_reduced_graphs, symbol_table))
    }
}
