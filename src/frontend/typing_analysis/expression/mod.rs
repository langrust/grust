use std::collections::HashMap;

use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::Expression, typedef::Typedef};
use crate::symbol_table::SymbolTable;

mod abstraction;
mod application;
mod array;
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
    /// use grustine::hir::expression::Expression;
    /// use grustine::common::{constant::Constant, location::Location};
    ///
    /// let mut errors = vec![];
    /// let global_context = HashMap::new();
    /// let elements_context = HashMap::new();
    /// let user_types_context = HashMap::new();
    /// let mut expression = Expression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: None,
    ///     location: Location::default(),
    /// };
    /// expression.typing(&global_context, &elements_context, &user_types_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            Expression::Constant { .. } => self.typing_constant(user_types_context, errors),
            Expression::Call { .. } => self.typing_call(symbol_table, errors),
            Expression::Application { .. } => self.typing_application(
                symbol_table,
                user_types_context,
                errors,
            ),
            Expression::Abstraction { .. } => {
                self.typing_abstraction(symbol_table, user_types_context, errors)
            }
            Expression::Structure { .. } => {
                self.typing_structure(symbol_table, user_types_context, errors)
            }
            Expression::Array { .. } => {
                self.typing_array(symbol_table, user_types_context, errors)
            }
            Expression::When { .. } => {
                self.typing_when(symbol_table, user_types_context, errors)
            }
            Expression::Match { .. } => {
                self.typing_match(symbol_table, user_types_context, errors)
            }
            Expression::FieldAccess { .. } => self.typing_field_access(
                symbol_table,
                user_types_context,
                errors,
            ),
            Expression::Map { .. } => {
                self.typing_map(symbol_table, user_types_context, errors)
            }
            Expression::Fold { .. } => {
                self.typing_fold(symbol_table, user_types_context, errors)
            }
            Expression::Sort { .. } => {
                self.typing_sort(symbol_table, user_types_context, errors)
            }
            Expression::Zip { .. } => {
                self.typing_zip(symbol_table, user_types_context, errors)
            }
            Expression::TupleElementAccess { .. } => self.typing_tuple_element_access(
                symbol_table,
                user_types_context,
                errors,
            ),
        }
    }

    /// Get the reference to the expression's typing.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::hir::expression::Expression;
    /// use grustine::common::{constant::Constant, location::Location, r#type::Type};
    ///
    /// let mut expression = Expression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Some(Type::Integer),
    ///     location: Location::default(),
    /// };
    /// let typing = expression.get_type().unwrap();
    /// ```
    pub fn get_type(&self) -> Option<&Type> {
        match self {
            Expression::Constant { typing, .. }
            | Expression::Call { typing, .. }
            | Expression::Application { typing, .. }
            | Expression::Abstraction { typing, .. }
            | Expression::Structure { typing, .. }
            | Expression::Array { typing, .. }
            | Expression::Match { typing, .. }
            | Expression::When { typing, .. }
            | Expression::FieldAccess { typing, .. }
            | Expression::TupleElementAccess { typing, .. }
            | Expression::Map { typing, .. }
            | Expression::Fold { typing, .. }
            | Expression::Sort { typing, .. }
            | Expression::Zip { typing, .. } => typing.as_ref(),
        }
    }

    /// Get the mutable reference to the expression's typing.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::hir::expression::Expression;
    /// use grustine::common::{constant::Constant, location::Location, r#type::Type};
    ///
    /// let mut expression = Expression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Some(Type::Integer),
    ///     location: Location::default(),
    /// };
    /// let typing = expression.get_type_mut().unwrap();
    /// ```
    pub fn get_type_mut(&mut self) -> Option<&mut Type> {
        match self {
            Expression::Constant { typing, .. }
            | Expression::Call { typing, .. }
            | Expression::Application { typing, .. }
            | Expression::Abstraction { typing, .. }
            | Expression::Structure { typing, .. }
            | Expression::Array { typing, .. }
            | Expression::Match { typing, .. }
            | Expression::When { typing, .. }
            | Expression::FieldAccess { typing, .. }
            | Expression::TupleElementAccess { typing, .. }
            | Expression::Map { typing, .. }
            | Expression::Fold { typing, .. }
            | Expression::Sort { typing, .. }
            | Expression::Zip { typing, .. } => typing.as_mut(),
        }
    }

    /// Get the expression's typing.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::hir::expression::Expression;
    /// use grustine::common::{constant::Constant, location::Location, r#type::Type};
    ///
    /// let mut expression = Expression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Some(Type::Integer),
    ///     location: Location::default(),
    /// };
    /// let typing = expression.get_type_owned().unwrap();
    /// ```
    pub fn get_type_owned(self) -> Option<Type> {
        match self {
            Expression::Constant { typing, .. }
            | Expression::Call { typing, .. }
            | Expression::Application { typing, .. }
            | Expression::Abstraction { typing, .. }
            | Expression::Structure { typing, .. }
            | Expression::Array { typing, .. }
            | Expression::Match { typing, .. }
            | Expression::When { typing, .. }
            | Expression::FieldAccess { typing, .. }
            | Expression::TupleElementAccess { typing, .. }
            | Expression::Map { typing, .. }
            | Expression::Fold { typing, .. }
            | Expression::Sort { typing, .. }
            | Expression::Zip { typing, .. } => typing,
        }
    }
}
