prelude! {
    frontend::TypeAnalysis,
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the application expression.
    pub fn typing_application(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        match self {
            // an application expression type is the result of the application
            // of the inputs types to the abstraction/function type
            hir::expr::Kind::Application {
                ref mut function_expression,
                ref mut inputs,
            } => {
                // type all inputs
                inputs
                    .iter_mut()
                    .map(|input| input.typing(symbol_table, errors))
                    .collect::<TRes<()>>()?;

                let input_types = inputs
                    .iter()
                    .map(|input| input.get_type().unwrap().clone())
                    .collect::<Vec<_>>();

                // type the function expression
                function_expression.typing(symbol_table, errors)?;

                // compute the application type
                let application_type = function_expression.get_type_mut().unwrap().apply(
                    input_types,
                    location.clone(),
                    errors,
                )?;

                Ok(application_type)
            }
            _ => unreachable!(),
        }
    }
}
