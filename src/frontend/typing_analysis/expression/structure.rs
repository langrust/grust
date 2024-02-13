use std::collections::HashMap;

use crate::hir::{expression::Expression, typedef::Typedef};
use crate::common::{context::Context, r#type::Type};
use crate::error::{Error, TerminationError};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the structure expression.
    pub fn typing_structure(
        &mut self,
        symbol_table: &mut SymbolTable,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // the type of the structure is the corresponding structure type
            // if fields match their expected types
            Expression::Structure {
                name,
                fields,
                typing,
                location,
            } => {
                // get the supposed structure type as the user defined it
                let user_type =
                    user_types_context.get_user_type_or_error(name, location.clone(), errors)?;

                match user_type {
                    Typedef::Structure { .. } => {
                        // type each field
                        fields
                            .iter_mut()
                            .map(|(_, expression)| {
                                expression.typing(
                                    symbol_table,
                                    user_types_context,
                                    errors,
                                )
                            })
                            .collect::<Vec<Result<(), TerminationError>>>()
                            .into_iter()
                            .collect::<Result<(), TerminationError>>()?;

                        // check that the structure is well defined
                        let well_defined_field =
                            |expression: &Expression,
                             field_type: &Type,
                             errors: &mut Vec<Error>| {
                                let expression_type = expression.get_type().unwrap();
                                expression_type.eq_check(field_type, location.clone(), errors)
                            };
                        user_type.well_defined_structure(fields, well_defined_field, errors)?;

                        *typing = Some(Type::Structure(name.clone()));
                        Ok(())
                    }
                    _ => {
                        let error = Error::ExpectStructure {
                            given_type: user_type.into_type(),
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
