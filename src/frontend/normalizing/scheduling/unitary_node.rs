use petgraph::algo::toposort;

use crate::{common::graph::neighbor::Label, hir::unitary_node::UnitaryNode};

impl UnitaryNode {
    /// Schedule equations.
    ///
    /// # Example.
    ///
    /// ```GR
    /// node test(v: int) {
    ///     out y: int = x-1
    ///     o_1: int = 0 fby x
    ///     x: int = v*2 + o_1
    /// }
    /// ```
    ///
    /// In the node above, signal `y` depends on the current value of `x`,
    /// `o_1` depends on the memory of `x` and `x` depends on `v` and `o_1`.
    /// The node is causal and should be scheduled as bellow:
    ///
    /// ```GR
    /// node test(v: int) {
    ///     o_1: int = 0 fby x  // depends on no current values of signals
    ///     x: int = v*2 + o_1  // depends on the computed value of `o_1` and given `v`
    ///     out y: int = x-1    // depends on the computed value of `x`
    /// }
    /// ```
    pub fn schedule(&mut self) {
        let graph = self
            .graph
            .get()
            .expect("node dependency graph should be computed");
        let mut subgraph = graph.clone();
        graph.all_edges().for_each(|(from, to, label)| match label {
            Label::Weight(0) => (),
            _ => assert_ne!(subgraph.remove_edge(from, to), Some(Label::Weight(0))),
        });

        let schedule = toposort(&subgraph, None).unwrap();

        let scheduled_equations = schedule
            .into_iter()
            .filter_map(|signal_id| {
                self.equations
                    .iter()
                    .position(|equation| equation.id == signal_id)
            })
            .map(|index| self.equations.get(index).unwrap().clone())
            .collect();

        self.equations = scheduled_equations;
    }
}
