use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::label::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a binop stream expression.
    pub fn compute_binop_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            // dependencies of binop are dependencies of the expressions
            ExpressionKind::Binop {
                left_expression,
                right_expression,
                ..
            } => {
                // get right and left expressions dependencies
                left_expression.compute_dependencies(symbol_table, nodes_reduced_graphs, errors)?;
                right_expression.compute_dependencies(
                    symbol_table,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut expression_dependencies = left_expression.get_dependencies().clone();
                let mut right_expression_dependencies = right_expression.get_dependencies().clone();
                expression_dependencies.append(&mut right_expression_dependencies);

                Ok(expression_dependencies)
            }
            _ => unreachable!(),
        }
    }
}
