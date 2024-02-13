use std::collections::HashMap;

use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::Expression, typedef::Typedef};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the match expression.
    pub fn typing_match(
        &mut self,
        symbol_table: &mut SymbolTable,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // the type of a match expression is the type of all branches expressions
            Expression::Match {
                expression,
                arms,
                typing,
                location,
            } => {
                expression.typing(symbol_table, user_types_context, errors)?;

                let expression_type = expression.get_type().unwrap();
                arms.iter_mut()
                    .map(|(pattern, optional_test_expression, _, arm_expression)| {
                        let optional_test_expression_typing_test = optional_test_expression
                            .as_mut()
                            .map_or(Ok(()), |expression| {
                                expression.typing(symbol_table, user_types_context, errors)?;
                                expression.get_type().unwrap().eq_check(
                                    &Type::Boolean,
                                    location.clone(),
                                    errors,
                                )
                            });

                        let arm_expression_typing_test =
                            arm_expression.typing(symbol_table, user_types_context, errors);

                        optional_test_expression_typing_test?;
                        arm_expression_typing_test
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                let first_type = arms[0].3.get_type().unwrap();
                arms.iter()
                    .map(|(_, _, _, arm_expression)| {
                        let arm_expression_type = arm_expression.get_type().unwrap();
                        arm_expression_type.eq_check(first_type, location.clone(), errors)
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                // todo: patterns should be exhaustive
                *typing = Some(first_type.clone());
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
