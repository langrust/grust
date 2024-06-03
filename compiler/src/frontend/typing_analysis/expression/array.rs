prelude! {
    frontend::TypeAnalysis,
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the array expression.
    pub fn typing_array(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        match self {
            // an array is composed of `n` elements of the same type `t` and
            // its type is `[t; n]`
            hir::expr::Kind::Array { ref mut elements } => {
                if elements.len() == 0 {
                    let error = Error::ExpectInput {
                        location: location.clone(),
                    };
                    errors.push(error);
                    return Err(TerminationError);
                }

                elements
                    .iter_mut()
                    .map(|element| element.typing(symbol_table, errors))
                    .collect::<TRes<()>>()?;

                let first_type = elements[0].get_type().unwrap(); // todo: manage zero element error
                elements
                    .iter()
                    .map(|element| {
                        let element_type = element.get_type().unwrap();
                        element_type.eq_check(first_type, location.clone(), errors)
                    })
                    .collect::<TRes<()>>()?;

                let array_type = Typ::array(first_type.clone(), elements.len());

                Ok(array_type)
            }
            _ => unreachable!(),
        }
    }
}
