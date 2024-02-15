use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::graph::color::Color;
use crate::common::graph::neighbor::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{node::Node, expression::{Expression, ExpressionKind}};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Compute dependencies of a fold stream expression.
    pub fn compute_fold_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_processus_manager: &mut HashMap<String, HashMap<&String, Color>>,
        nodes_graphs: &mut HashMap<String, DiGraphMap<usize, Label>>,
        nodes_reduced_graphs: &mut HashMap<String, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            // dependencies of fold are dependencies of the folded expression
            ExpressionKind::Fold {
                expression,
                initialization_expression,
                ..
            } => {
                // get folded expression dependencies
                expression.compute_dependencies(
                    nodes_context,
                    nodes_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut expression_dependencies = expression.get_dependencies().clone();

                // get initialization expression dependencies
                initialization_expression.compute_dependencies(
                    nodes_context,
                    nodes_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut initialization_expression_dependencies =
                    initialization_expression.get_dependencies().clone();

                expression_dependencies.append(&mut initialization_expression_dependencies);
                // push in fold dependencies
                self.dependencies.set(expression_dependencies);

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

