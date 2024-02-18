use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::graph::color::Color;
use crate::common::graph::neighbor::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, node::Node, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a function application stream expression.
    pub fn compute_function_application_dependencies(
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
            // dependencies of function application are dependencies of its inputs
            ExpressionKind::Application { inputs, .. } => {
                // propagate dependencies computation
                inputs
                    .iter()
                    .map(|input_expression| {
                        input_expression.compute_dependencies(
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

                Ok(inputs
                    .iter()
                    .flat_map(|input_expression| input_expression.get_dependencies().clone())
                    .collect())
            }
            _ => unreachable!(),
        }
    }
}
