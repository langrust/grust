use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::graph::color::Color;
use crate::common::graph::neighbor::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::{Expression, ExpressionKind}};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Compute dependencies of a function application stream expression.
    pub fn compute_function_application_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_processus_manager: &mut HashMap<String, HashMap<&String, Color>>,
        nodes_graphs: &mut HashMap<String, DiGraphMap<usize, Label>>,
        nodes_reduced_graphs: &mut HashMap<String, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            // dependencies of function application are dependencies of its inputs
            ExpressionKind::Application {
                inputs,
                ..
            } => {
                // propagate dependencies computation
                inputs
                    .iter()
                    .map(|input_expression| {
                        input_expression.compute_dependencies(
                            nodes_context,
                            nodes_processus_manager,
                            nodes_graphs,
                            nodes_reduced_graphs,
                            errors,
                        )
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<_, _>>()?;

                // set dependencies
                self.dependencies.set(
                    inputs
                        .iter()
                        .flat_map(|input_expression| input_expression.get_dependencies().clone())
                        .collect(),
                );

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
