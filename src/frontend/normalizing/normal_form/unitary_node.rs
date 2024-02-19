use std::collections::HashMap;

use petgraph::graphmap::{DiGraphMap, GraphMap};

use crate::{
    common::graph::neighbor::Label,
    hir::{identifier_creator::IdentifierCreator, statement::Statement, unitary_node::UnitaryNode},
    symbol_table::SymbolTable,
};

impl UnitaryNode {
    /// Change HIR unitary node into a normal form.
    ///
    /// The normal form of an unitary node is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    /// }
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     x_1: int = v*2;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// ```
    pub fn normal_form(
        &mut self,
        nodes_reduced_graphs: &HashMap<usize, DiGraphMap<usize, Label>>,
        symbol_table: &mut SymbolTable,
    ) {
        let mut identifier_creator = IdentifierCreator::from(self.get_signals_name(symbol_table));

        let UnitaryNode { statements, .. } = self;

        *statements = statements
            .clone()
            .into_iter()
            .flat_map(|equation| {
                equation.normal_form(nodes_reduced_graphs, &mut identifier_creator, symbol_table)
            })
            .collect();

        // add a dependency graph to the unitary node
        let mut graph = GraphMap::new();
        self.get_signals_id().iter().for_each(|signal_id| {
            graph.add_node(*signal_id);
        });
        self.statements.iter().for_each(
            |Statement {
                 id: from,
                 expression,
                 ..
             }| {
                for (to, weight) in expression.get_dependencies() {
                    graph.add_edge(*from, *to, Label::Weight(*weight));
                }
            },
        );
        self.graph = graph;
    }
}
