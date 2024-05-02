use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::label::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a zip stream expression.
    pub fn compute_zip_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            // dependencies of zip are dependencies of its arrays
            ExpressionKind::Zip { arrays, .. } => {
                // propagate dependencies computation
                arrays
                    .iter()
                    .map(|array_expression| {
                        array_expression.compute_dependencies(
                            symbol_table,
                            nodes_reduced_graphs,
                            errors,
                        )
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<_, _>>()?;

                Ok(arrays
                    .iter()
                    .flat_map(|array_expression| array_expression.get_dependencies().clone())
                    .collect())
            }
            _ => unreachable!(),
        }
    }
}
