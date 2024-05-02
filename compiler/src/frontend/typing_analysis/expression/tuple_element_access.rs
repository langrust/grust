use crate::common::{location::Location, r#type::Type};
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::expression::ExpressionKind;
use crate::symbol_table::SymbolTable;

impl<E> ExpressionKind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Type] to the tuple element access expression.
    pub fn typing_tuple_element_access(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Type, TerminationError> {
        match self {
            ExpressionKind::TupleElementAccess {
                ref mut expression,
                ref element_number,
            } => {
                expression.typing(symbol_table, errors)?;

                match expression.get_type().unwrap() {
                    Type::Tuple(elements_type) => {
                        let option_element_type = elements_type.get(*element_number);
                        if let Some(element_type) = option_element_type {
                            Ok(element_type.clone())
                        } else {
                            let error = Error::IndexOutOfBounds {
                                location: location.clone(),
                            };
                            errors.push(error);
                            Err(TerminationError)
                        }
                    }
                    given_type => {
                        let error = Error::ExpectTuple {
                            given_type: given_type.clone(),
                            location: location.clone(),
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
