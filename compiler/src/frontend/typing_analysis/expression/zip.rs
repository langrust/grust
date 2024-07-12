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
                    Typ::Array { size: n, .. } => Ok(n),
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
                        Typ::Array { ty, size: n, .. } if n == length => Ok(*ty.clone()),
                        Typ::Array { size: n, .. } => {
                            let error = Error::IncompatibleLength {
                                given_length: n.base10_parse().unwrap(),
                                expected_length: length.base10_parse().unwrap(),
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
                    Typ::array(Typ::tuple(tuple_types), length.base10_parse().unwrap())
                } else {
                    Typ::array(
                        tuple_types.get(0).unwrap().clone(),
                        length.base10_parse().unwrap(),
                    )
                };

                Ok(array_type)
            }
            _ => unreachable!(),
        }
    }
}
