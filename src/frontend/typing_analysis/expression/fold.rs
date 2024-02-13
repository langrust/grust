use std::collections::HashMap;

use crate::hir::{expression::Expression, typedef::Typedef};
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the fold expression.
    pub fn typing_fold(
        &mut self,
        symbol_table: &mut SymbolTable,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            Expression::Fold {
                expression,
                initialization_expression,
                function_expression,
                typing,
                location,
            } => {
                // type the expression
                expression.typing(symbol_table, user_types_context, errors)?;

                // verify it is an array
                match expression.get_type().unwrap() {
                    Type::Array(element_type, _) => {
                        // type the initialization expression
                        initialization_expression.typing(
                            symbol_table,
                            user_types_context,
                            errors,
                        )?;
                        let initialization_type = initialization_expression.get_type().unwrap();

                        // type the function expression
                        function_expression.typing(
                            symbol_table,
                            user_types_context,
                            errors,
                        )?;
                        let function_type = function_expression.get_type_mut().unwrap();

                        // apply the function type to the type of the initialization and array's elements
                        let new_type = function_type.apply(
                            vec![initialization_type.clone(), *element_type.clone()],
                            location.clone(),
                            errors,
                        )?;

                        // check the new type is equal to the initialization type
                        new_type.eq_check(initialization_type, location.clone(), errors)?;

                        *typing = Some(new_type);
                        Ok(())
                    }
                    given_type => {
                        let error = Error::ExpectArray {
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
