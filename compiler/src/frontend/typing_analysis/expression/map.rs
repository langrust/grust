prelude! {
    frontend::typing_analysis::TypeAnalysis,
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the map expression.
    pub fn typing_map(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        match self {
            hir::expr::Kind::Map {
                ref mut expression,
                ref mut function_expression,
            } => {
                // type the expression
                expression.typing(symbol_table, errors)?;

                // verify it is an array
                match expression.get_type().unwrap() {
                    Typ::Array(element_type, size) => {
                        // type the function expression
                        function_expression.typing(symbol_table, errors)?;
                        let function_type = function_expression.get_type_mut().unwrap();

                        // apply the function type to the type of array's elements
                        let new_element_type = function_type.apply(
                            vec![*element_type.clone()],
                            location.clone(),
                            errors,
                        )?;

                        Ok(Typ::Array(Box::new(new_element_type), *size))
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
