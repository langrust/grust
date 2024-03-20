use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::color::Color;
use crate::common::label::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, 
stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a match stream expression.
    pub fn compute_match_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_processus_manager: &mut HashMap<usize, HashMap<usize, Color>>,
        nodes_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            // dependencies of match are dependencies of matched expression and
            // dependencies of arms (without new signals defined in patterns)
            ExpressionKind::Match {
                expression, arms, ..
            } => {
                // compute arms dependencies
                let mut arms_dependencies = arms
                    .iter()
                    .map(|(pattern, bound, _, arm_expression)| {
                        // get local signals defined in pattern
                        let local_signals = pattern.local_identifiers();

                        // get arm expression dependencies
                        arm_expression.compute_dependencies(
                            symbol_table,
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
                            .collect::<Vec<(usize, Label)>>();

                        // get bound dependencies
                        let mut bound_dependencies =
                            bound.as_ref().map_or(Ok(vec![]), |bound_expression| {
                                bound_expression.compute_dependencies(
                                    symbol_table,
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
                    .collect::<Result<Vec<Vec<(usize, Label)>>, TerminationError>>()?
                    .into_iter()
                    .flatten()
                    .collect::<Vec<(usize, Label)>>();

                // get matched expression dependencies
                expression.compute_dependencies(
                    symbol_table,
                    nodes_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut expression_dependencies = expression.get_dependencies().clone();

                arms_dependencies.append(&mut expression_dependencies);
                Ok(arms_dependencies)
            }
            _ => unreachable!(),
        }
    }
}
