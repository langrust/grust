use petgraph::algo::toposort;

use crate::{
    common::label::Label,
    error::{Error, TerminationError},
    hir::node::Node,
    symbol_table::SymbolTable,
};

impl Node {
    /// Check the causality of the node.
    ///
    /// # Example
    /// The folowing simple node is causal, there is no causality loop.
    /// ```GR
    /// node causal_node1(i: int) {
    ///     out o: int = x;
    ///     x: int = i;
    /// }
    /// ```
    ///
    /// The next node is causal as well, `x` does not depends on `o` but depends
    /// on the memory of `o`. Then there is no causality loop.
    /// ```GR
    /// node causal_node2() {
    ///     out o: int = x;
    ///     x: int = 0 fby o;
    /// }
    /// ```
    ///
    /// But the node that follows is not causal, `o` depends on `x` which depends
    /// on `o`. Values of signals can not be determined, then the compilation
    /// raises a causality error.
    /// ```GR
    /// node not_causal_node(i: int) {
    ///     out o: int = x;
    ///     x: int = o;
    /// }
    /// ```
    pub fn causal(
        &self,
        symbol_table: &SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        // construct node's subgraph containing only 0-label weight
        let graph = &self.graph;
        let mut subgraph = graph.clone();
        graph.all_edges().for_each(|(from, to, label)| match label {
            Label::Weight(0) => (),
            _ => debug_assert_ne!(subgraph.remove_edge(from, to), Some(Label::Weight(0))),
        });

        // if a schedule exists, then the node is causal
        let _ = toposort(&subgraph, None).map_err(|signal| {
            let error = Error::NotCausalSignal {
                node: symbol_table.get_name(self.id).clone(),
                signal: symbol_table.get_name(signal.node_id()).clone(),
                location: self.location.clone(),
            };
            errors.push(error);
            TerminationError
        })?;

        Ok(())
    }
}
