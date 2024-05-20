use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::color::Color;
use crate::common::label::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of an array stream expression.
    pub fn compute_array_dependencies(
        &self,
        graph: &mut DiGraphMap<usize, Label>,
        symbol_table: &SymbolTable,
        processus_manager: &mut HashMap<usize, Color>,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            // dependencies of array are dependencies of its elements
            ExpressionKind::Array { elements } => {
                // propagate dependencies computation
                elements
                    .iter()
                    .map(|element_expression| {
                        element_expression.compute_dependencies(graph, symbol_table, processus_manager, 
                            nodes_reduced_graphs,
                            errors,
                        )
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<_, _>>()?;

                Ok(elements
                    .iter()
                    .flat_map(|element_expression| element_expression.get_dependencies().clone())
                    .collect())
            }
            _ => unreachable!(),
        }
    }
}
