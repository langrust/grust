use std::collections::BTreeMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::color::Color;
use crate::common::label::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, node::Node, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of an tuple stream expression.
    pub fn compute_tuple_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_context: &BTreeMap<usize, Node>,
        nodes_processus_manager: &mut BTreeMap<usize, BTreeMap<usize, Color>>,
        nodes_reduced_processus_manager: &mut BTreeMap<usize, BTreeMap<usize, Color>>,
        nodes_graphs: &mut BTreeMap<usize, DiGraphMap<usize, Label>>,
        nodes_reduced_graphs: &mut BTreeMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            // dependencies of tuple are dependencies of its elements
            ExpressionKind::Tuple { elements } => {
                // propagate dependencies computation
                elements
                    .iter()
                    .map(|element_expression| {
                        element_expression.compute_dependencies(
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

                Ok(elements
                    .iter()
                    .flat_map(|element_expression| element_expression.get_dependencies().clone())
                    .collect())
            }
            _ => unreachable!(),
        }
    }
}
