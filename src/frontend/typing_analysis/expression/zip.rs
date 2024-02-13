use std::collections::HashMap;

use crate::hir::{expression::Expression, typedef::Typedef};
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the zip expression.
    pub fn typing_zip(
        &mut self,
        symbol_table: &mut SymbolTable,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            Expression::Zip {
                arrays,
                typing,
                location,
            } => {
                if arrays.len() == 0 {
                    let error = Error::ExpectInput {
                        location: location.clone(),
                    };
                    errors.push(error);
                    return Err(TerminationError);
                }

                arrays
                    .iter_mut()
                    .map(|array| {
                        array.typing(symbol_table, user_types_context, errors)
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                let length = match arrays[0].get_type().unwrap() {
                    Type::Array(_, n) => Ok(n),
                    ty => {
                        let error = Error::ExpectArray {
                            given_type: ty.clone(),
                            location: location.clone(),
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }?;
                let tuple_types = arrays
                    .iter()
                    .map(|array| match array.get_type().unwrap() {
                        Type::Array(ty, n) if n == length => Ok(*ty.clone()),
                        Type::Array(_, n) => {
                            let error = Error::IncompatibleLength {
                                given_length: *n,
                                expected_length: *length,
                                location: location.clone(),
                            };
                            errors.push(error);
                            Err(TerminationError)
                        }
                        ty => {
                            let error = Error::ExpectArray {
                                given_type: ty.clone(),
                                location: location.clone(),
                            };
                            errors.push(error);
                            Err(TerminationError)
                        }
                    })
                    .collect::<Vec<Result<Type, TerminationError>>>()
                    .into_iter()
                    .collect::<Result<Vec<Type>, TerminationError>>()?;

                let array_type = if tuple_types.len() > 1 {
                    Type::Array(Box::new(Type::Tuple(tuple_types)), *length)
                } else {
                    Type::Array(Box::new(tuple_types.get(0).unwrap().clone()), *length)
                };

                *typing = Some(array_type);
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
