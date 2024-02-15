use crate::error::{Error, TerminationError};
use crate::hir::expression::{Expression, ExpressionKind};
use crate::symbol_table::SymbolTable;

mod abstraction;
mod application;
mod array;
mod enumeration;
mod call;
mod constant;
mod field_access;
mod fold;
mod map;
mod r#match;
mod sort;
mod structure;
mod tuple_element_access;
mod when;
mod zip;

impl Expression {
    /// Add a [Type] to the expression.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::hir::ExpressionKind::Expression;
    /// use grustine::common::{constant::Constant, location::Location};
    ///
    /// let mut errors = vec![];
    /// let global_context = HashMap::new();
    /// let elements_context = HashMap::new();
    /// let user_types_context = HashMap::new();
    /// let mut expression = ExpressionKind::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: None,
    ///     location: Location::default(),
    /// };
    /// expression.typing(&global_context, &elements_context, & &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            ExpressionKind::Constant { .. } => self.typing_constant(symbol_table, errors),
            ExpressionKind::Identifier { .. } => self.typing_call(symbol_table, errors),
            ExpressionKind::Application { .. } => self.typing_application(symbol_table, errors),
            ExpressionKind::Abstraction { .. } => self.typing_abstraction(symbol_table, errors),
            ExpressionKind::Structure { .. } => self.typing_structure(symbol_table, errors),
            ExpressionKind::Array { .. } => self.typing_array(symbol_table, errors),
            ExpressionKind::When { .. } => self.typing_when(symbol_table, errors),
            ExpressionKind::Match { .. } => self.typing_match(symbol_table, errors),
            ExpressionKind::FieldAccess { .. } => self.typing_field_access(symbol_table, errors),
            ExpressionKind::Map { .. } => self.typing_map(symbol_table, errors),
            ExpressionKind::Fold { .. } => self.typing_fold(symbol_table, errors),
            ExpressionKind::Sort { .. } => self.typing_sort(symbol_table, errors),
            ExpressionKind::Zip { .. } => self.typing_zip(symbol_table, errors),
            ExpressionKind::TupleElementAccess { .. } => {
                self.typing_tuple_element_access(symbol_table, errors)
            }
            ExpressionKind::Enumeration { enum_id, elem_id } => todo!(),
        }
    }
}
