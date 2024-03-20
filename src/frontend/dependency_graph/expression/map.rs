use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::label::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a map stream expression.
    pub fn compute_map_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            // dependencies of map are dependencies of the mapped expression
            ExpressionKind::Map { expression, .. } => {
                // get mapped expression dependencies
                expression.compute_dependencies(symbol_table, nodes_reduced_graphs, errors)?;
                let expression_dependencies = expression.get_dependencies().clone();

                Ok(expression_dependencies)
            }
            _ => unreachable!(),
        }
    }
}
