use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::graph::color::Color;
use crate::common::graph::neighbor::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{node::Node, expression::{Expression, ExpressionKind}};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Compute dependencies of a sort stream expression.
    pub fn compute_sort_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_processus_manager: &mut HashMap<String, HashMap<&String, Color>>,
        nodes_graphs: &mut HashMap<String, DiGraphMap<usize, Label>>,
        nodes_reduced_graphs: &mut HashMap<String, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            // dependencies of sort are dependencies of the sorted expression
            ExpressionKind::Sort {
                expression,
                ..
            } => {
                // get sorted expression dependencies
                expression.compute_dependencies(
                    nodes_context,
                    nodes_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let expression_dependencies = expression.get_dependencies().clone();

                // push in sort dependencies
                self.dependencies.set(expression_dependencies);

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

