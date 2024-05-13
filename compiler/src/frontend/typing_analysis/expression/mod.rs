use crate::common::{location::Location, r#type::Type};
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::expression::{Expression, ExpressionKind};
use crate::symbol_table::SymbolTable;

mod abstraction;
mod application;
mod array;
mod binop;
mod constant;
mod enumeration;
mod field_access;
mod fold;
mod identifier;
mod if_then_else;
mod map;
mod r#match;
mod sort;
mod structure;
mod tuple;
mod tuple_element_access;
mod unop;
mod when;
mod zip;

impl<E> ExpressionKind<E>
where
    E: TypeAnalysis,
{
    /// Tries to type the given construct.
    pub fn typing(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Type, TerminationError> {
        match self {
            ExpressionKind::Constant { .. } => self.typing_constant(),
            ExpressionKind::Identifier { .. } => self.typing_identifier(symbol_table),
            ExpressionKind::Unop { .. } => self.typing_unop(location, symbol_table, errors),
            ExpressionKind::Binop { .. } => self.typing_binop(location, symbol_table, errors),
            ExpressionKind::IfThenElse { .. } => {
                self.typing_if_then_else(location, symbol_table, errors)
            }
            ExpressionKind::Application { .. } => {
                self.typing_application(location, symbol_table, errors)
            }
            ExpressionKind::Abstraction { .. } => self.typing_abstraction(symbol_table, errors),
            ExpressionKind::Structure { .. } => {
                self.typing_structure(location, symbol_table, errors)
            }
            ExpressionKind::Array { .. } => self.typing_array(location, symbol_table, errors),
            ExpressionKind::Tuple { .. } => self.typing_tuple(symbol_table, errors),
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
            ExpressionKind::Enumeration { .. } => self.typing_enumeration(symbol_table),
        }
    }
}

impl TypeAnalysis for Expression {
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
