use crate::common::{location::Location, r#type::Type};
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::expression::{Expression, ExpressionKind};
use crate::symbol_table::SymbolTable;

mod abstraction;
mod application;
mod array;
mod call;
mod constant;
mod enumeration;
mod field_access;
mod fold;
mod map;
mod r#match;
mod sort;
mod structure;
mod tuple_element_access;
mod when;
mod zip;

impl<E> ExpressionKind<E>
where
    E: TypeAnalysis,
{
    pub fn typing(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Type, TerminationError> {
        match self {
            ExpressionKind::Constant { .. } => self.typing_constant(),
            ExpressionKind::Identifier { .. } => self.typing_call(symbol_table),
            ExpressionKind::Application { .. } => {
                self.typing_application(location, symbol_table, errors)
            }
            ExpressionKind::Abstraction { .. } => self.typing_abstraction(symbol_table, errors),
            ExpressionKind::Structure { .. } => {
                self.typing_structure(location, symbol_table, errors)
            }
            ExpressionKind::Array { .. } => self.typing_array(location, symbol_table, errors),
            ExpressionKind::When { .. } => self.typing_when(location, symbol_table, errors),
            ExpressionKind::Match { .. } => self.typing_match(location, symbol_table, errors),
            ExpressionKind::FieldAccess { .. } => {
                self.typing_field_access(location, symbol_table, errors)
            }
            ExpressionKind::Map { .. } => self.typing_map(location, symbol_table, errors),
            ExpressionKind::Fold { .. } => self.typing_fold(location, symbol_table, errors),
            ExpressionKind::Sort { .. } => self.typing_sort(location, symbol_table, errors),
            ExpressionKind::Zip { .. } => self.typing_zip(location, symbol_table, errors),
            ExpressionKind::TupleElementAccess { .. } => {
                self.typing_tuple_element_access(location, symbol_table, errors)
            }
            ExpressionKind::Enumeration { .. } => todo!(),
        }
    }
}

impl TypeAnalysis for Expression {
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
    fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        self.typing = Some(self.kind.typing(&self.location, symbol_table, errors)?);
        Ok(())
    }
    fn get_type(&self) -> Option<&Type> {
        self.typing.as_ref()
    }
    fn get_type_mut(&mut self) -> Option<&mut Type> {
        self.typing.as_mut()
    }
}
