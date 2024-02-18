use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::graph::color::Color;
use crate::common::graph::neighbor::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, node::Node, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a when stream expression.
    pub fn compute_when_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_context: &HashMap<usize, Node>,
        nodes_processus_manager: &mut HashMap<usize, HashMap<usize, Color>>,
        nodes_reduced_processus_manager: &mut HashMap<usize, HashMap<usize, Color>>,
        nodes_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, usize)>, TerminationError> {
        match self {
            // dependencies of when are dependencies of the optional expression
            // plus present and default expressions (without the new local signal)
            ExpressionKind::When {
                id: local_signal,
                option,
                present,
                default,
                ..
            } => {
                // get dependencies of optional expression
                option.compute_dependencies(
                    symbol_table,
                    nodes_context,
                    nodes_processus_manager,
                    nodes_reduced_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut option_dependencies = option.get_dependencies().clone();

                // get dependencies of present expression without local signal
                present.compute_dependencies(
                    symbol_table,
                    nodes_context,
                    nodes_processus_manager,
                    nodes_reduced_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut present_dependencies = present
                    .get_dependencies()
                    .clone()
                    .into_iter()
                    .filter(|(signal, _)| !signal.eq(local_signal))
                    .collect();

                // get dependencies of default expression without local signal
                default.compute_dependencies(
                    symbol_table,
                    nodes_context,
                    nodes_processus_manager,
                    nodes_reduced_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut default_dependencies = default
                    .get_dependencies()
                    .clone()
                    .into_iter()
                    .filter(|(signal, _)| !signal.eq(local_signal))
                    .collect();

                // push all dependencies in optional dependencies
                option_dependencies.append(&mut present_dependencies);
                option_dependencies.append(&mut default_dependencies);
                Ok(option_dependencies)
            }
            _ => unreachable!(),
        }
    }
}
