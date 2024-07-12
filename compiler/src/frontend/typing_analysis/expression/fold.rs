prelude! {
    frontend::TypeAnalysis,
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the fold expression.
    pub fn typing_fold(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        match self {
            hir::expr::Kind::Fold {
                ref mut expression,
                ref mut initialization_expression,
                ref mut function_expression,
            } => {
                // type the expression
                expression.typing(symbol_table, errors)?;

                // verify it is an array
                match expression.get_type().unwrap() {
                    Typ::Array {
                        ty: element_type, ..
                    } => {
                        // type the initialization expression
                        initialization_expression.typing(symbol_table, errors)?;
                        let initialization_type = initialization_expression.get_type().unwrap();

                        // type the function expression
                        function_expression.typing(symbol_table, errors)?;
                        let function_type = function_expression.get_type_mut().unwrap();

                        // apply the function type to the type of the initialization and array's elements
                        let new_type = function_type.apply(
                            vec![initialization_type.clone(), *element_type.clone()],
                            location.clone(),
                            errors,
                        )?;

                        // check the new type is equal to the initialization type
                        new_type.eq_check(initialization_type, location.clone(), errors)?;

                        Ok(new_type)
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
