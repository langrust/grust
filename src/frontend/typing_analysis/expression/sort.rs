use std::collections::HashMap;

use crate::hir::{expression::Expression, typedef::Typedef};
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the sort expression.
    pub fn typing_sort(
        &mut self,
        symbol_table: &mut SymbolTable,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            Expression::Sort {
                expression,
                function_expression,
                typing,
                location,
            } => {
                // type the expression
                expression.typing(symbol_table, user_types_context, errors)?;

                // verify it is an array
                match expression.get_type().unwrap() {
                    Type::Array(element_type, size) => {
                        // type the function expression
                        function_expression.typing(
                            symbol_table,
                            user_types_context,
                            errors,
                        )?;
                        let function_type = function_expression.get_type_mut().unwrap();

                        // check it is a sorting function: (element_type, element_type) -> int
                        function_type.eq_check(
                            &Type::Abstract(
                                vec![*element_type.clone(), *element_type.clone()],
                                Box::new(Type::Integer),
                            ),
                            location.clone(),
                            errors,
                        )?;

                        *typing = Some(Type::Array(element_type.clone(), *size));
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
