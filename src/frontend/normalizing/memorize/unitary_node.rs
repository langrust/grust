use petgraph::graphmap::GraphMap;

use crate::{
    common::graph::neighbor::Label,
    hir::{
        identifier_creator::IdentifierCreator, memory::Memory, once_cell::OnceCell,
        statement::Statement, unitary_node::UnitaryNode,
    },
    symbol_table::SymbolTable,
};

impl UnitaryNode {
    /// Create memory for HIR unitary nodes.
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
    ///
    /// This example is tested in source.
    pub fn memorize(&mut self, symbol_table: &mut SymbolTable) {
        let mut identifier_creator = IdentifierCreator::from(self.get_signals_name(symbol_table));
        let mut memory = Memory::new();

        self.statements.iter_mut().for_each(|statement| {
            statement.memorize(
                &mut identifier_creator,
                &mut memory,
                &mut self.contract,
                symbol_table,
            )
        });

        self.memory = memory;

        // add a dependency graph to the unitary node
        let mut graph = GraphMap::new();
        self.get_signals_id().iter().for_each(|signal_id| {
            graph.add_node(*signal_id);
        });
        self.memory.buffers.keys().for_each(|signal_id| {
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
        self.graph = OnceCell::from(graph);
    }
}
