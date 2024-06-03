prelude! {
    frontend::typing_analysis::TypeAnalysis,
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the tuple element access expression.
    pub fn typing_tuple_element_access(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        match self {
            hir::expr::Kind::TupleElementAccess {
                ref mut expression,
                ref element_number,
            } => {
                expression.typing(symbol_table, errors)?;

                match expression.get_type().unwrap() {
                    Typ::Tuple(elements_type) => {
                        let option_element_type = elements_type.get(*element_number);
                        if let Some(element_type) = option_element_type {
                            Ok(element_type.clone())
                        } else {
                            let error = Error::IndexOutOfBounds {
                                location: location.clone(),
                            };
                            errors.push(error);
                            Err(TerminationError)
                        }
                    }
                    given_type => {
                        let error = Error::ExpectTuple {
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
