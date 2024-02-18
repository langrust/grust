use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::graph::color::Color;
use crate::common::graph::neighbor::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, node::Node, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a structure stream expression.
    pub fn compute_structure_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_context: &HashMap<usize, Node>,
        nodes_processus_manager: &mut HashMap<usize, HashMap<usize, Color>>,
        nodes_reduced_processus_manager: &mut HashMap<usize, HashMap<usize, Color>>,
        nodes_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, usize)>, TerminationError> {
        match self {
            // dependencies of structure are dependencies of its fields
            ExpressionKind::Structure { fields, .. } => {
                // propagate dependencies computation
                fields
                    .iter()
                    .map(|(_, field_expression)| {
                        field_expression.compute_dependencies(
                            symbol_table,
                            nodes_context,
                            nodes_processus_manager,
                            nodes_reduced_processus_manager,
                            nodes_graphs,
                            nodes_reduced_graphs,
                            errors,
                        )
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<_, _>>()?;

                Ok(fields
                    .iter()
                    .flat_map(|(_, field_expression)| field_expression.get_dependencies().clone())
                    .collect())
            }
            _ => unreachable!(),
        }
    }
}
