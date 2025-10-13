use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::label::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

mod abstraction;
mod application;
mod array;
mod constant;
mod enumeration;
mod field_access;
mod fold;
mod identifier;
mod map;
mod r#match;
mod sort;
mod structure;
mod tuple;
mod tuple_element_access;
mod when;
mod zip;

impl ExpressionKind<StreamExpression> {
    /// Get nodes applications identifiers.
    pub fn get_called_nodes(&self) -> Vec<usize> {
        match &self {
            ExpressionKind::Constant { .. }
            | ExpressionKind::Identifier { .. }
            | ExpressionKind::Enumeration { .. } => vec![],
            ExpressionKind::Application {
                function_expression,
                inputs,
            } => {
                let mut nodes = inputs
                    .iter()
                    .flat_map(|expression| expression.get_called_nodes())
                    .collect::<Vec<_>>();
                let mut other_nodes = function_expression.get_called_nodes();
                nodes.append(&mut other_nodes);
                nodes
            }
            ExpressionKind::Abstraction { expression, .. } => expression.get_called_nodes(),
            ExpressionKind::Structure { fields, .. } => fields
                .iter()
                .flat_map(|(_, expression)| expression.get_called_nodes())
                .collect::<Vec<_>>(),
            ExpressionKind::Array { elements } => elements
                .iter()
                .flat_map(|expression| expression.get_called_nodes())
                .collect::<Vec<_>>(),
            ExpressionKind::Tuple { elements } => elements
                .iter()
                .flat_map(|expression| expression.get_called_nodes())
                .collect::<Vec<_>>(),
            ExpressionKind::Match { expression, arms } => {
                let mut nodes = expression.get_called_nodes();
                let mut other_nodes = arms
                    .iter()
                    .flat_map(|(_, bound, body, expression)| {
                        debug_assert!(body.is_empty());
                        let mut nodes = expression.get_called_nodes();
                        let mut other_nodes = bound
                            .as_ref()
                            .map_or(vec![], |expression| expression.get_called_nodes());
                        nodes.append(&mut other_nodes);
                        nodes
                    })
                    .collect::<Vec<_>>();
                nodes.append(&mut other_nodes);
                nodes
            }
            ExpressionKind::When {
                option,
                present,
                present_body,
                default,
                default_body,
                ..
            } => {
                debug_assert!(present_body.is_empty());
                debug_assert!(default_body.is_empty());
                let mut nodes = option.get_called_nodes();
                let mut other_nodes = present.get_called_nodes();
                nodes.append(&mut other_nodes);
                let mut other_nodes = default.get_called_nodes();
                nodes.append(&mut other_nodes);
                nodes
            }
            ExpressionKind::FieldAccess { expression, .. } => expression.get_called_nodes(),
            ExpressionKind::TupleElementAccess { expression, .. } => expression.get_called_nodes(),
            ExpressionKind::Map {
                expression,
                function_expression,
            } => {
                let mut nodes = expression.get_called_nodes();
                let mut other_nodes = function_expression.get_called_nodes();
                nodes.append(&mut other_nodes);
                nodes
            }
            ExpressionKind::Fold {
                expression,
                initialization_expression,
                function_expression,
            } => {
                let mut nodes = expression.get_called_nodes();
                let mut other_nodes = initialization_expression.get_called_nodes();
                nodes.append(&mut other_nodes);
                let mut other_nodes = function_expression.get_called_nodes();
                nodes.append(&mut other_nodes);
                nodes
            }
            ExpressionKind::Sort {
                expression,
                function_expression,
            } => {
                let mut nodes = expression.get_called_nodes();
                let mut other_nodes = function_expression.get_called_nodes();
                nodes.append(&mut other_nodes);
                nodes
            }
            ExpressionKind::Zip { arrays } => arrays
                .iter()
                .flat_map(|expression| expression.get_called_nodes())
                .collect::<Vec<_>>(),
        }
    }

    /// Compute dependencies of a stream expression.
    ///
    /// # Example
    ///
    /// Considering the following node:
    ///
    /// ```GR
    /// node my_node(x: int, y: int) {
    ///     out o: int = 0 fby z;
    ///     z: int = 1 fby (x + y);
    /// }
    /// ```
    ///
    /// The stream expression `my_node(f(x), 1).o` depends on the signal `x` with
    /// a dependency label weight of 2. Indeed, the expression depends on the memory
    /// of the memory of `x` (the signal is behind 2 fby operations).
    pub fn compute_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            ExpressionKind::Constant { .. } => self.compute_constant_dependencies(),
            ExpressionKind::Identifier { .. } => self.compute_identifier_dependencies(symbol_table),
            ExpressionKind::Abstraction { .. } => self.compute_abstraction_dependencies(),
            ExpressionKind::Enumeration { .. } => self.compute_enumeration_dependencies(),
            ExpressionKind::Application { .. } => self.compute_function_application_dependencies(
                symbol_table,
                nodes_reduced_graphs,
                errors,
            ),
            ExpressionKind::Structure { .. } => {
                self.compute_structure_dependencies(symbol_table, nodes_reduced_graphs, errors)
            }
            ExpressionKind::Array { .. } => {
                self.compute_array_dependencies(symbol_table, nodes_reduced_graphs, errors)
            }
            ExpressionKind::Tuple { .. } => {
                self.compute_tuple_dependencies(symbol_table, nodes_reduced_graphs, errors)
            }
            ExpressionKind::Match { .. } => {
                self.compute_match_dependencies(symbol_table, nodes_reduced_graphs, errors)
            }
            ExpressionKind::When { .. } => {
                self.compute_when_dependencies(symbol_table, nodes_reduced_graphs, errors)
            }
            ExpressionKind::FieldAccess { .. } => {
                self.compute_field_access_dependencies(symbol_table, nodes_reduced_graphs, errors)
            }
            ExpressionKind::TupleElementAccess { .. } => self
                .compute_tuple_element_access_dependencies(
                    symbol_table,
                    nodes_reduced_graphs,
                    errors,
                ),
            ExpressionKind::Map { .. } => {
                self.compute_map_dependencies(symbol_table, nodes_reduced_graphs, errors)
            }
            ExpressionKind::Fold { .. } => {
                self.compute_fold_dependencies(symbol_table, nodes_reduced_graphs, errors)
            }
            ExpressionKind::Sort { .. } => {
                self.compute_sort_dependencies(symbol_table, nodes_reduced_graphs, errors)
            }
            ExpressionKind::Zip { .. } => {
                self.compute_zip_dependencies(symbol_table, nodes_reduced_graphs, errors)
            }
        }
    }
}
