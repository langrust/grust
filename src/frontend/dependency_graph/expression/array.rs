use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::graph::color::Color;
use crate::common::graph::neighbor::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{node::Node, expression::{Expression, ExpressionKind}};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Compute dependencies of an array stream expression.
    pub fn compute_array_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_processus_manager: &mut HashMap<String, HashMap<&String, Color>>,
        nodes_graphs: &mut HashMap<String, DiGraphMap<usize, Label>>,
        nodes_reduced_graphs: &mut HashMap<String, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            // dependencies of array are dependencies of its elements
            ExpressionKind::Array {
                elements,
                ..
            } => {
                // propagate dependencies computation
                elements
                    .iter()
                    .map(|element_expression| {
                        element_expression.compute_dependencies(
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
                    elements
                        .iter()
                        .flat_map(|element_expression| {
                            element_expression.get_dependencies().clone()
                        })
                        .collect(),
                );

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

