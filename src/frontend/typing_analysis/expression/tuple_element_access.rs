use std::collections::HashMap;

use crate::hir::{expression::Expression, typedef::Typedef};
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the tuple element access expression.
    pub fn typing_tuple_element_access(
        &mut self,
        symbol_table: &mut SymbolTable,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            Expression::TupleElementAccess {
                expression,
                element_number,
                typing,
                location,
            } => {
                expression.typing(symbol_table, user_types_context, errors)?;

                match expression.get_type().unwrap() {
                    Type::Tuple(elements_type) => {
                        let option_element_type = elements_type.get(*element_number);
                        if let Some(element_type) = option_element_type {
                            *typing = Some(element_type.clone());
                            Ok(())
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
