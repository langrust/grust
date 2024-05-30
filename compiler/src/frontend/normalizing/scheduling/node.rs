use petgraph::algo::toposort;

prelude! {
    common::{label::Label, HashMap},
    hir::{
        expression::ExpressionKind,
        node::Node,
        statement::Statement,
        stream_expression::{StreamExpression, StreamExpressionKind},
    },
}

impl Node {
    /// Schedule statements.
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
        // get subgraph with only direct dependencies
        let mut subgraph = self.graph.clone();
        self.graph
            .all_edges()
            .for_each(|(from, to, label)| match label {
                Label::Weight(0) => (),
                _ => debug_assert_ne!(subgraph.remove_edge(from, to), Some(Label::Weight(0))),
            });

        // topological sorting
        let mut schedule = toposort(&subgraph, None).unwrap();
        schedule.reverse();

        // construct map from signal_id to their position in the schedule
        let signals_order = schedule
            .into_iter()
            .enumerate()
            .map(|(order, signal_id)| (signal_id, order))
            .collect::<HashMap<_, _>>();
        let compare = |statement: &Statement<StreamExpression>| {
            statement
                .pattern
                .identifiers()
                .into_iter()
                .map(|signal_id| signals_order.get(&signal_id).unwrap())
                .min()
                .unwrap()
        };

        // sort statements
        self.statements.sort_by_key(compare);
        self.statements.iter_mut().for_each(|statement| {
            match &mut statement.expression.kind {
                StreamExpressionKind::Expression { expression } => match expression {
                    ExpressionKind::Match { arms, .. } => arms
                        .iter_mut()
                        .for_each(|(_, _, statements, _)| statements.sort_by_key(compare)),
                    _ => (),
                },
                _ => (),
            };
        })
    }
}
