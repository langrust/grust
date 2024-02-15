use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::graph::color::Color;
use crate::common::graph::neighbor::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{node::Node, expression::{Expression, ExpressionKind}};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Compute dependencies of a match stream expression.
    pub fn compute_match_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_processus_manager: &mut HashMap<String, HashMap<&String, Color>>,
        nodes_graphs: &mut HashMap<String, DiGraphMap<usize, Label>>,
        nodes_reduced_graphs: &mut HashMap<String, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            // dependencies of match are dependencies of matched expression and
            // dependencies of arms (without new signals defined in patterns)
            ExpressionKind::Match {
                expression,
                arms,
                ..
            } => {
                // compute arms dependencies
                let mut arms_dependencies = arms
                    .iter()
                    .map(|(pattern, bound, _, arm_expression)| {
                        // get local signals defined in pattern
                        let local_signals = pattern.local_identifiers();

                        // get arm expression dependencies
                        arm_expression.compute_dependencies(
                            nodes_context,
                            nodes_processus_manager,
                            nodes_graphs,
                            nodes_reduced_graphs,
                            errors,
                        )?;
                        let mut arm_dependencies = arm_expression
                            .get_dependencies()
                            .clone()
                            .into_iter()
                            .filter(|(signal, _)| !local_signals.contains(signal))
                            .collect::<Vec<(String, usize)>>();

                        // get bound dependencies
                        let mut bound_dependencies =
                            bound.as_ref().map_or(Ok(vec![]), |bound_expression| {
                                bound_expression.compute_dependencies(
                                    nodes_context,
                                    nodes_processus_manager,
                                    nodes_graphs,
                                    nodes_reduced_graphs,
                                    errors,
                                )?;

                                Ok(bound_expression
                                    .get_dependencies()
                                    .clone()
                                    .into_iter()
                                    .filter(|(signal, _)| !local_signals.contains(signal))
                                    .collect())
                            })?;

                        // push all dependencies in arm dependencies
                        arm_dependencies.append(&mut bound_dependencies);

                        // return arm dependencies
                        Ok(arm_dependencies)
                    })
                    .collect::<Result<Vec<Vec<(String, usize)>>, TerminationError>>()?
                    .into_iter()
                    .flatten()
                    .collect::<Vec<(String, usize)>>();

                // get matched expression dependencies
                expression.compute_dependencies(
                    nodes_context,
                    nodes_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut expression_dependencies = expression.get_dependencies().clone();

                // push all dependencies in arms dependencies
                arms_dependencies.append(&mut expression_dependencies);
                self.dependencies.set(arms_dependencies);

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

