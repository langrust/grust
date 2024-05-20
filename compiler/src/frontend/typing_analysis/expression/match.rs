use crate::common::{location::Location, r#type::Type};
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::expression::ExpressionKind;
use crate::symbol_table::SymbolTable;

impl<E> ExpressionKind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Type] to the match expression.
    pub fn typing_match(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Type, TerminationError> {
        match self {
            // the type of a match expression is the type of all branches expressions
            ExpressionKind::Match {
                ref mut expression,
                ref mut arms,
            } => {
                expression.typing(symbol_table, errors)?;

                let expression_type = expression.get_type().unwrap();

                arms.iter_mut()
                    .map(
                        |(pattern, optional_test_expression, body, arm_expression)| {
                            // check it matches pattern type
                            pattern.typing(expression_type, symbol_table, errors)?;

                            optional_test_expression
                                .as_mut()
                                .map_or(Ok(()), |expression| {
                                    expression.typing(symbol_table, errors)?;
                                    expression.get_type().unwrap().eq_check(
                                        &Type::Boolean,
                                        location.clone(),
                                        errors,
                                    )
                                })?;

                            // set types for every pattern
                            body.iter_mut()
                                .map(|statement| {
                                    statement
                                        .pattern
                                        .construct_statement_type(symbol_table, errors)
                                })
                                .collect::<Vec<Result<(), TerminationError>>>()
                                .into_iter()
                                .collect::<Result<(), TerminationError>>()?;

                            // type all equations
                            body.iter_mut()
                                .map(|statement| statement.typing(symbol_table, errors))
                                .collect::<Vec<Result<(), TerminationError>>>()
                                .into_iter()
                                .collect::<Result<(), TerminationError>>()?;

                            arm_expression.typing(symbol_table, errors)
                        },
                    )
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
                Ok(first_type.clone())
            }
            _ => unreachable!(),
        }
    }
}
