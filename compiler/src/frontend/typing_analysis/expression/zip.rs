prelude! {
    frontend::typing_analysis::TypeAnalysis,
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the zip expression.
    pub fn typing_zip(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        match self {
            hir::expr::Kind::Zip { ref mut arrays } => {
                if arrays.len() == 0 {
                    let error = Error::ExpectInput {
                        location: location.clone(),
                    };
                    errors.push(error);
                    return Err(TerminationError);
                }

                arrays
                    .iter_mut()
                    .map(|array| array.typing(symbol_table, errors))
                    .collect::<TRes<()>>()?;

                let length = match arrays[0].get_type().unwrap() {
                    Typ::Array(_, n) => Ok(n),
                    ty => {
                        let error = Error::ExpectArray {
                            given_type: ty.clone(),
                            location: location.clone(),
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }?;
                let tuple_types = arrays
                    .iter()
                    .map(|array| match array.get_type().unwrap() {
                        Typ::Array(ty, n) if n == length => Ok(*ty.clone()),
                        Typ::Array(_, n) => {
                            let error = Error::IncompatibleLength {
                                given_length: *n,
                                expected_length: *length,
                                location: location.clone(),
                            };
                            errors.push(error);
                            Err(TerminationError)
                        }
                        ty => {
                            let error = Error::ExpectArray {
                                given_type: ty.clone(),
                                location: location.clone(),
                            };
                            errors.push(error);
                            Err(TerminationError)
                        }
                    })
                    .collect::<TRes<Vec<Typ>>>()?;

                let array_type = if tuple_types.len() > 1 {
                    Typ::Array(Box::new(Typ::Tuple(tuple_types)), *length)
                } else {
                    Typ::Array(Box::new(tuple_types.get(0).unwrap().clone()), *length)
                };

                Ok(array_type)
            }
            _ => unreachable!(),
        }
    }
}
