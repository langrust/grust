prelude! {
    operator::OtherOperator,
    frontend::TypeAnalysis,
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the if_then_else expression.
    pub fn typing_if_then_else(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        match self {
            // an if_then_else expression type is the result of the if_then_else
            // of the inputs types to the abstraction/function type
            hir::expr::Kind::IfThenElse {
                expression,
                true_expression,
                false_expression,
            } => {
                // get expressions type
                expression.typing(symbol_table, errors)?;
                let expression_type = expression.get_type().unwrap().clone();
                true_expression.typing(symbol_table, errors)?;
                let true_expression_type = true_expression.get_type().unwrap().clone();
                false_expression.typing(symbol_table, errors)?;
                let false_expression_type = false_expression.get_type().unwrap().clone();

                // get if_then_else type
                let mut if_then_else_type = OtherOperator::IfThenElse.get_type();

                if_then_else_type.apply(
                    vec![expression_type, true_expression_type, false_expression_type],
                    location.clone(),
                    errors,
                )
            }
            _ => unreachable!(),
        }
    }
}
