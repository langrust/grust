use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::hir::expression::{Expression, ExpressionKind};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the match expression.
    pub fn typing_match(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            // the type of a match expression is the type of all branches expressions
            ExpressionKind::Match {
                ref mut expression,
                ref mut arms,
            } => {
                expression.typing(symbol_table, errors)?;

                let expression_type = expression.get_type().unwrap();
                arms.iter_mut()
                    .map(|(pattern, optional_test_expression, _, arm_expression)| {
                        let optional_test_expression_typing_test = optional_test_expression
                            .as_mut()
                            .map_or(Ok(()), |expression| {
                                expression.typing(symbol_table, errors)?;
                                expression.get_type().unwrap().eq_check(
                                    &Type::Boolean,
                                    self.location.clone(),
                                    errors,
                                )
                            });

                        let arm_expression_typing_test =
                            arm_expression.typing(symbol_table, errors);

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
                        arm_expression_type.eq_check(first_type, self.location.clone(), errors)
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                // todo: patterns should be exhaustive
                self.typing = Some(first_type.clone());
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
