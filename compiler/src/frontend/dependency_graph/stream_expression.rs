prelude! {
    graph::*,
    hir::stream,
    frontend::ctx::*,
}

impl stream::Expr {
    /// Get nodes applications identifiers.
    pub fn get_called_nodes(&self) -> Vec<usize> {
        match &self.kind {
            stream::Kind::Event { .. } => vec![],
            stream::Kind::Expression { expression } => expression.get_called_nodes(),
            stream::Kind::FollowedBy { expression, .. } => expression.get_called_nodes(),
            stream::Kind::NodeApplication {
                called_node_id,
                inputs,
                ..
            } => {
                let mut nodes = inputs
                    .iter()
                    .flat_map(|(_, expression)| expression.get_called_nodes())
                    .collect::<Vec<_>>();
                nodes.push(*called_node_id);
                nodes
            }
        }
    }

    /// Compute dependencies of a stream expression.
    ///
    /// # Example
    ///
    /// Considering the following node:
    ///
    /// ```GR
    /// node my_node(x: int, y: int) {
    ///     out o: int = 0 fby z;
    ///     z: int = 1 fby (x + y);
    /// }
    /// ```
    ///
    /// The stream expression `my_node(f(x), 1).o` depends on the signal `x` with
    /// a dependency label weight of 2. Indeed, the expression depends on the memory
    /// of the memory of `x` (the signal is behind 2 fby operations).
    pub fn compute_dependencies(&self, ctx: &mut GraphProcCtx) -> TRes<()> {
        match &self.kind {
            stream::Kind::Event { event_id } => {
                self.dependencies.set(vec![(*event_id, Label::Weight(0))]);
                Ok(())
            }
            stream::Kind::FollowedBy {
                ref constant,
                ref expression,
            } => {
                // propagate dependencies computation in expression
                expression.compute_dependencies(ctx)?;
                // dependencies with the memory delay
                let dependencies = expression
                    .get_dependencies()
                    .clone()
                    .into_iter()
                    .map(|(id, label)| (id, label.increment()))
                    .collect();

                // constant should not have dependencies
                constant.compute_dependencies(ctx)?;
                debug_assert!({ constant.get_dependencies().is_empty() });

                self.dependencies.set(dependencies);
                Ok(())
            }
            stream::Kind::NodeApplication {
                ref called_node_id,
                ref inputs,
                ..
            } => {
                // function "dependencies to inputs" and "input expressions's dependencies"
                // of node application
                self.dependencies.set(
                    inputs
                        .iter()
                        .map(|(input_id, input_expression)| {
                            // compute input expression dependencies
                            input_expression.compute_dependencies(ctx)?;

                            let symbol_table = ctx.symbol_table;
                            // get reduced graph (graph with only inputs/outputs signals)
                            let reduced_graph = ctx.reduced_graphs.get_mut(called_node_id).unwrap();

                            // for each node's output, get dependencies from output to inputs
                            let dependencies = symbol_table
                                .get_node_outputs(*called_node_id)
                                .iter()
                                .flat_map(|(_, output_signal)| {
                                    reduced_graph.edge_weight(*output_signal, *input_id).map_or(
                                        vec![],
                                        |label1| {
                                            input_expression
                                                .get_dependencies()
                                                .clone()
                                                .into_iter()
                                                .map(|(id, label2)| (id, label1.add(&label2)))
                                                .collect()
                                        },
                                    )
                                })
                                .collect();

                            Ok(dependencies)
                        })
                        .collect::<TRes<Vec<Vec<(usize, Label)>>>>()?
                        .into_iter()
                        .flatten()
                        .collect::<Vec<(usize, Label)>>(),
                );

                Ok(())
            }
            stream::Kind::Expression { expression } => {
                self.dependencies.set(expression.compute_dependencies(ctx)?);
                Ok(())
            }
        }
    }
}
