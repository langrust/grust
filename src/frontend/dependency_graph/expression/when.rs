use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::graph::color::Color;
use crate::common::graph::neighbor::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{node::Node, expression::{Expression, ExpressionKind}};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Compute dependencies of a when stream expression.
    pub fn compute_when_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_processus_manager: &mut HashMap<String, HashMap<&String, Color>>,
        nodes_graphs: &mut HashMap<String, DiGraphMap<usize, Label>>,
        nodes_reduced_graphs: &mut HashMap<String, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
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
                    nodes_context,
                    nodes_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut option_dependencies = option.get_dependencies().clone();

                // get dependencies of present expression without local signal
                present.compute_dependencies(
                    nodes_context,
                    nodes_processus_manager,
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
                    nodes_context,
                    nodes_processus_manager,
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
                self.dependencies.set(option_dependencies);

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

