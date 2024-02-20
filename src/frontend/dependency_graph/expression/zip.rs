use std::collections::BTreeMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::graph::color::Color;
use crate::common::graph::neighbor::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, node::Node, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a zip stream expression.
    pub fn compute_zip_dependencies(
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
            // dependencies of zip are dependencies of its arrays
            ExpressionKind::Zip { arrays, .. } => {
                // propagate dependencies computation
                arrays
                    .iter()
                    .map(|array_expression| {
                        array_expression.compute_dependencies(
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

                Ok(arrays
                    .iter()
                    .flat_map(|array_expression| array_expression.get_dependencies().clone())
                    .collect())
            }
            _ => unreachable!(),
        }
    }
}
