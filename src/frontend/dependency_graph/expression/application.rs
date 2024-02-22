use std::collections::BTreeMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::color::Color;
use crate::common::label::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, node::Node, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a function application stream expression.
    pub fn compute_function_application_dependencies(
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
            // dependencies of function application are dependencies of its inputs
            ExpressionKind::Application {
                function_expression,
                inputs,
            } => {
                // propagate dependencies computation
                function_expression.compute_dependencies(
                    symbol_table,
                    nodes_context,
                    nodes_processus_manager,
                    nodes_reduced_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
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

                // combine dependencies
                let mut dependencies = function_expression.get_dependencies().clone();
                let mut inputs_dependencies = inputs
                    .iter()
                    .flat_map(|input_expression| input_expression.get_dependencies().clone())
                    .collect::<Vec<_>>();
                dependencies.append(&mut inputs_dependencies);

                Ok(dependencies)
            }
            _ => unreachable!(),
        }
    }
}
