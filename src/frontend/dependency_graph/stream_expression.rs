use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::graph::color::Color;
use crate::common::graph::neighbor::Label;
use crate::error::{Error, TerminationError};
use crate::hir::stream_expression::StreamExpressionKind;
use crate::hir::{node::Node, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl StreamExpression {
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
    /// a dependency depth of 2. Indeed, the expression depends on the memory
    /// of the memory of `x` (the signal is behind 2 fby operations).
    pub fn compute_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_processus_manager: &mut HashMap<usize, HashMap<usize, Color>>,
        nodes_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match &self.kind {
            StreamExpressionKind::FollowedBy { ref expression, .. } => {
                // propagate dependencies computation
                expression.compute_dependencies(
                    symbol_table,
                    nodes_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;

                // set dependencies with the memory delay
                self.dependencies.set(
                    expression
                        .get_dependencies()
                        .clone()
                        .into_iter()
                        .map(|(id, depth)| (id, depth + 1))
                        .collect(),
                );

                Ok(())
            }
            StreamExpressionKind::NodeApplication {
                ref node_id,
                ref inputs,
                ref output_id,
            } => {
                // get called node
                let node: Node = todo!("get node");

                // create local reduced graphs (because only complete for the called signal)
                let mut local_nodes_reduced_graphs = nodes_reduced_graphs.clone();
                // let mut local_nodes_processus_manager = nodes_processus_manager.clone(); // TODO: see if this is important

                // add dependencies to inputs in the local graphs
                node.add_signal_inputs_dependencies(
                    output_id,
                    symbol_table,
                    nodes_processus_manager,
                    nodes_graphs,
                    &mut local_nodes_reduced_graphs,
                    errors,
                )?;

                // get both "real reduced graph" and "local reduced graph" of called node
                let local_reduced_graph = local_nodes_reduced_graphs.get(node_id).unwrap();
                let reduced_graph = nodes_reduced_graphs.get_mut(node_id).unwrap();

                // store computed dependencies (in "local reduced graph") into "real reduced graph"
                local_reduced_graph
                    .edges(*output_id)
                    .for_each(|(_, id, label)| {
                        reduced_graph.add_edge(*output_id, id, label.clone());
                    });

                // function "dependencies to inputs" and "input expressions's dependencies"
                // of node application
                self.dependencies.set(
                    inputs
                        .iter()
                        .map(|(input_id, input_expression)| {
                            input_expression.compute_dependencies(
                                symbol_table,
                                nodes_processus_manager,
                                nodes_graphs,
                                nodes_reduced_graphs,
                                errors,
                            )?;
                            Ok(local_reduced_graph
                                .edge_weight(*output_id, *input_id)
                                .map_or(Ok(vec![]), |label| {
                                    match label {
                                        Label::Contract => Ok(vec![]), // TODO: do we loose the CREUSOT dependence with the input?
                                        Label::Weight(weight) => Ok(input_expression
                                            .get_dependencies()
                                            .clone()
                                            .into_iter()
                                            .map(|(id, depth)| (id, depth + weight))
                                            .collect()),
                                    }
                                })?)
                        })
                        .collect::<Result<Vec<Vec<(usize, usize)>>, TerminationError>>()?
                        .into_iter()
                        .flatten()
                        .collect::<Vec<(usize, usize)>>(),
                );

                Ok(())
            }
            StreamExpressionKind::Expression { expression } => {
                self.dependencies.set(expression.compute_dependencies(
                    symbol_table,
                    nodes_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?);
                Ok(())
            }
            StreamExpressionKind::UnitaryNodeApplication { .. } => unreachable!(),
        }
    }
}
