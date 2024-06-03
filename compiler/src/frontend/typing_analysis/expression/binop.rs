prelude! {
    frontend::TypeAnalysis,
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the binop expression.
    pub fn typing_binop(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        match self {
            // an binop expression type is the result of the binop
            // of the inputs types to the abstraction/function type
            hir::expr::Kind::Binop {
                op,
                left_expression,
                right_expression,
            } => {
                // get expressions type
                left_expression.typing(symbol_table, errors)?;
                let left_expression_type = left_expression.get_type().unwrap().clone();
                right_expression.typing(symbol_table, errors)?;
                let right_expression_type = right_expression.get_type().unwrap().clone();

                // get binop type
                let mut binop_type = op.get_type();

                binop_type.apply(
                    vec![left_expression_type, right_expression_type],
                    location.clone(),
                    errors,
                )
            }
            _ => unreachable!(),
        }
    }
}
