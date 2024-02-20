use std::collections::BTreeMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::graph::color::Color;
use crate::common::graph::neighbor::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, node::Node, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a fold stream expression.
    pub fn compute_fold_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_context: &BTreeMap<usize, Node>,
        nodes_processus_manager: &mut BTreeMap<usize, BTreeMap<usize, Color>>,
        nodes_reduced_processus_manager: &mut BTreeMap<usize, BTreeMap<usize, Color>>,
        nodes_graphs: &mut BTreeMap<usize, DiGraphMap<usize, Label>>,
        nodes_reduced_graphs: &mut BTreeMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, usize)>, TerminationError> {
        match self {
            // dependencies of fold are dependencies of the folded expression
            ExpressionKind::Fold {
                expression,
                initialization_expression,
                ..
            } => {
                // get folded expression dependencies
                expression.compute_dependencies(
                    symbol_table,
                    nodes_context,
                    nodes_processus_manager,
                    nodes_reduced_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut expression_dependencies = expression.get_dependencies().clone();

                // get initialization expression dependencies
                initialization_expression.compute_dependencies(
                    symbol_table,
                    nodes_context,
                    nodes_processus_manager,
                    nodes_reduced_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut initialization_expression_dependencies =
                    initialization_expression.get_dependencies().clone();

                expression_dependencies.append(&mut initialization_expression_dependencies);
                Ok(expression_dependencies)
            }
            _ => unreachable!(),
        }
    }
}
