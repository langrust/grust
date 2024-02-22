use std::collections::BTreeMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::color::Color;
use crate::common::label::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::ExpressionKind, node::Node, stream_expression::StreamExpression};
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
mod tuple_element_access;
mod when;
mod zip;

impl ExpressionKind<StreamExpression> {
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
        nodes_context: &BTreeMap<usize, Node>,
        nodes_processus_manager: &mut BTreeMap<usize, BTreeMap<usize, Color>>,
        nodes_reduced_processus_manager: &mut BTreeMap<usize, BTreeMap<usize, Color>>,
        nodes_graphs: &mut BTreeMap<usize, DiGraphMap<usize, Label>>,
        nodes_reduced_graphs: &mut BTreeMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            ExpressionKind::Constant { .. } => self.compute_constant_dependencies(),
            ExpressionKind::Identifier { .. } => self.compute_identifier_dependencies(symbol_table),
            ExpressionKind::Abstraction { .. } => self.compute_abstraction_dependencies(),
            ExpressionKind::Enumeration { .. } => self.compute_enumeration_dependencies(),
            ExpressionKind::Application { .. } => self.compute_function_application_dependencies(
                symbol_table,
                nodes_context,
                nodes_processus_manager,
                nodes_reduced_processus_manager,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            ExpressionKind::Structure { .. } => self.compute_structure_dependencies(
                symbol_table,
                nodes_context,
                nodes_processus_manager,
                nodes_reduced_processus_manager,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            ExpressionKind::Array { .. } => self.compute_array_dependencies(
                symbol_table,
                nodes_context,
                nodes_processus_manager,
                nodes_reduced_processus_manager,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            ExpressionKind::Match { .. } => self.compute_match_dependencies(
                symbol_table,
                nodes_context,
                nodes_processus_manager,
                nodes_reduced_processus_manager,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            ExpressionKind::When { .. } => self.compute_when_dependencies(
                symbol_table,
                nodes_context,
                nodes_processus_manager,
                nodes_reduced_processus_manager,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            ExpressionKind::FieldAccess { .. } => self.compute_field_access_dependencies(
                symbol_table,
                nodes_context,
                nodes_processus_manager,
                nodes_reduced_processus_manager,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            ExpressionKind::TupleElementAccess { .. } => self
                .compute_tuple_element_access_dependencies(
                    symbol_table,
                    nodes_context,
                    nodes_processus_manager,
                    nodes_reduced_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                ),
            ExpressionKind::Map { .. } => self.compute_map_dependencies(
                symbol_table,
                nodes_context,
                nodes_processus_manager,
                nodes_reduced_processus_manager,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            ExpressionKind::Fold { .. } => self.compute_fold_dependencies(
                symbol_table,
                nodes_context,
                nodes_processus_manager,
                nodes_reduced_processus_manager,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            ExpressionKind::Sort { .. } => self.compute_sort_dependencies(
                symbol_table,
                nodes_context,
                nodes_processus_manager,
                nodes_reduced_processus_manager,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            ExpressionKind::Zip { .. } => self.compute_zip_dependencies(
                symbol_table,
                nodes_context,
                nodes_processus_manager,
                nodes_reduced_processus_manager,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
        }
    }
}
