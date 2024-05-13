use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::label::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a ifthenelse stream expression.
    pub fn compute_ifthenelse_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            // dependencies of ifthenelse are dependencies of the expressions
            ExpressionKind::IfThenElse {
                expression,
                true_expression,
                false_expression,
            } => {
                // get right and left expressions dependencies
                expression.compute_dependencies(symbol_table, nodes_reduced_graphs, errors)?;
                true_expression.compute_dependencies(symbol_table, nodes_reduced_graphs, errors)?;
                false_expression.compute_dependencies(
                    symbol_table,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut expression_dependencies = expression.get_dependencies().clone();
                let mut true_expression_dependencies = true_expression.get_dependencies().clone();
                let mut false_expression_dependencies = false_expression.get_dependencies().clone();
                expression_dependencies.append(&mut true_expression_dependencies);
                expression_dependencies.append(&mut false_expression_dependencies);

                Ok(expression_dependencies)
            }
            _ => unreachable!(),
        }
    }
}
