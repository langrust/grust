use std::collections::HashMap;

use crate::hir::{expression::Expression, typedef::Typedef};
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the field access expression.
    pub fn typing_field_access(
        &mut self,
        symbol_table: &mut SymbolTable,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            Expression::FieldAccess {
                expression,
                field,
                typing,
                location,
            } => {
                expression.typing(symbol_table, user_types_context, errors)?;

                match expression.get_type().unwrap() {
                    Type::Structure(type_id) => match user_types_context.get(type_id).unwrap() {
                        Typedef::Structure { fields, .. } => {
                            let option_field_type = fields
                                .iter()
                                .filter(|(f, _)| f == field)
                                .map(|(_, t)| t.clone())
                                .next();
                            if let Some(field_type) = option_field_type {
                                *typing = Some(field_type);
                                Ok(())
                            } else {
                                let error = Error::UnknownField {
                                    structure_name: type_id.clone(),
                                    field_name: field.clone(),
                                    location: location.clone(),
                                };
                                errors.push(error);
                                Err(TerminationError)
                            }
                        }
                        user_type => {
                            let error = Error::ExpectStructure {
                                given_type: user_type.into_type(),
                                location: location.clone(),
                            };
                            errors.push(error);
                            Err(TerminationError)
                        }
                    },
                    given_type => {
                        let error = Error::ExpectStructure {
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
