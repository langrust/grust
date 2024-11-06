prelude! {
    graph::Label,
}

use petgraph::algo::toposort;

impl hir::Component {
    /// Check the causality of the node.
    ///
    /// # Example
    ///
    /// The following simple node is causal, there is no causality loop.
    ///
    /// ```GR
    /// node causal_node1(i: int) {
    ///     out o: int = x;
    ///     x: int = i;
    /// }
    /// ```
    ///
    /// The next node is causal as well, `x` does not depends on `o` but depends on the memory of
    /// `o`. Then there is no causality loop.
    ///
    /// ```GR
    /// node causal_node2() {
    ///     out o: int = x;
    ///     x: int = 0 fby o;
    /// }
    /// ```
    ///
    /// But the node that follows is not causal, `o` depends on `x` which depends on `o`. Values of
    /// signals can not be determined, then the compilation raises a causality error.
    ///
    /// ```GR
    /// node not_causal_node(i: int) {
    ///     out o: int = x;
    ///     x: int = o;
    /// }
    /// ```
    pub fn causal(&self, symbol_table: &SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        // construct node's subgraph containing only 0-label weight
        let graph = self.get_graph();
        let mut subgraph = graph.clone();
        graph.all_edges().for_each(|(from, to, label)| match label {
            Label::Weight(0) => (),
            _ => {
                let _label = subgraph.remove_edge(from, to);
                debug_assert_ne!(_label, Some(Label::Weight(0)))
            }
        });

        // if a schedule exists, then the node is causal
        let _ = toposort(&subgraph, None).map_err(|signal| {
            let error = Error::NotCausalSignal {
                signal: symbol_table.get_name(signal.node_id()).clone(),
                location: self.get_location().clone(),
            };
            errors.push(error);
            TerminationError
        })?;

        Ok(())
    }
}

impl hir::File {
    /// Check the causality of the file.
    ///
    /// # Example
    ///
    /// The following file is causal, there is no causality loop.
    ///
    /// ```GR
    /// node causal_node(i: int) {
    ///     out o: int = x;
    ///     x: int = i;
    /// }
    ///
    /// component causal_component() {
    ///     out o: int = x;
    ///     x: int = 0 fby o;
    /// }
    /// ```
    ///
    /// But the file that follows is not causal. In the node `not_causal_node`, signal`o` depends on
    /// `x` which depends on `o`. Values of signals can not be determined, then the compilation
    /// raises a causality error.
    ///
    /// ```GR
    /// node not_causal_node(i: int) {
    ///     out o: int = x;
    ///     x: int = o;
    /// }
    ///
    /// component causal_component() {
    ///     out o: int = x;
    ///     x: int = 0 fby o;
    /// }
    /// ```
    pub fn causality_analysis(
        &self,
        symbol_table: &SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<()> {
        // check causality for each node
        self.components
            .iter()
            .map(|node| node.causal(symbol_table, errors))
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<TRes<_>>()
    }
}
