use std::collections::HashMap;

use crate::hir::{expression::Expression, typedef::Typedef};
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the when expression.
    pub fn typing_when(
        &mut self,
        symbol_table: &mut SymbolTable,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // the type of a when expression is the type of both the default and
            // the present expressions
            Expression::When {
                id,
                option,
                present,
                default,
                typing,
                location,
                ..
            } => {
                option.typing(symbol_table, user_types_context, errors)?;

                let option_type = option.get_type().unwrap();
                match option_type {
                    Type::Option(unwraped_type) => {
                        // TODO: add type to id
                        present.typing(
                            symbol_table,
                            user_types_context,
                            errors,
                        )?;
                        default.typing(
                            symbol_table,
                            user_types_context,
                            errors,
                        )?;

                        let present_type = present.get_type().unwrap();
                        let default_type = default.get_type().unwrap();

                        *typing = Some(present_type.clone());
                        default_type.eq_check(present_type, location.clone(), errors)
                    }
                    _ => {
                        let error = Error::ExpectOption {
                            given_type: option_type.clone(),
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
