use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::color::Color;
use crate::common::label::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a sort stream expression.
    pub fn compute_sort_dependencies(
        &self,
        graph: &mut DiGraphMap<usize, Label>,
        symbol_table: &SymbolTable,
        processus_manager: &mut HashMap<usize, Color>,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            // dependencies of sort are dependencies of the sorted expression
            ExpressionKind::Sort { expression, .. } => {
                // get sorted expression dependencies
                expression.compute_dependencies(graph, symbol_table, processus_manager, nodes_reduced_graphs, errors)?;
                let expression_dependencies = expression.get_dependencies().clone();

                Ok(expression_dependencies)
            }
            _ => unreachable!(),
        }
    }
}
