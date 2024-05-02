use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::label::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a fold stream expression.
    pub fn compute_fold_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            // dependencies of fold are dependencies of the folded expression
            ExpressionKind::Fold {
                expression,
                initialization_expression,
                ..
            } => {
                // get folded expression dependencies
                expression.compute_dependencies(symbol_table, nodes_reduced_graphs, errors)?;
                let mut expression_dependencies = expression.get_dependencies().clone();

                // get initialization expression dependencies
                initialization_expression.compute_dependencies(
                    symbol_table,
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
