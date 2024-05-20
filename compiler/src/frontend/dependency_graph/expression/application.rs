use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::color::Color;
use crate::common::label::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a function application stream expression.
    pub fn compute_function_application_dependencies(
        &self,
        graph: &mut DiGraphMap<usize, Label>,
        symbol_table: &SymbolTable,
        processus_manager: &mut HashMap<usize, Color>,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
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
                    graph,
                    symbol_table,
                    processus_manager,
                    nodes_reduced_graphs,
                    errors,
                )?;
                inputs
                    .iter()
                    .map(|input_expression| {
                        input_expression.compute_dependencies(
                            graph,
                            symbol_table,
                            processus_manager,
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
