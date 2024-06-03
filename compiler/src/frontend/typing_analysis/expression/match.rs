prelude! {
    frontend::typing_analysis::TypeAnalysis,
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the match expression.
    pub fn typing_match(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        match self {
            // the type of a match expression is the type of all branches expressions
            hir::expr::Kind::Match {
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
                                        &Typ::Boolean,
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
                                .collect::<TRes<()>>()?;

                            // type all equations
                            body.iter_mut()
                                .map(|statement| statement.typing(symbol_table, errors))
                                .collect::<TRes<()>>()?;

                            arm_expression.typing(symbol_table, errors)
                        },
                    )
                    .collect::<TRes<()>>()?;

                let first_type = arms[0].3.get_type().unwrap();
                arms.iter()
                    .map(|(_, _, _, arm_expression)| {
                        let arm_expression_type = arm_expression.get_type().unwrap();
                        arm_expression_type.eq_check(first_type, location.clone(), errors)
                    })
                    .collect::<TRes<()>>()?;

                // todo: patterns should be exhaustive
                Ok(first_type.clone())
            }
            _ => unreachable!(),
        }
    }
}
