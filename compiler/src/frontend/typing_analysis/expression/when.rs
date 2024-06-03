prelude! {
    frontend::typing_analysis::TypeAnalysis,
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the when expression.
    pub fn typing_when(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        match self {
            // the type of a when expression is the type of both the default and
            // the present expressions
            hir::expr::Kind::When {
                ref id,
                ref mut option,
                ref mut present,
                ref mut default,
                ..
            } => {
                option.typing(symbol_table, errors)?;

                let option_type = option.get_type().unwrap();
                match option_type {
                    Typ::SMEvent(unwraped_type) => {
                        symbol_table.set_type(*id, *unwraped_type.clone());
                        present.typing(symbol_table, errors)?;
                        default.typing(symbol_table, errors)?;

                        let present_type = present.get_type().unwrap();
                        let default_type = default.get_type().unwrap();

                        default_type.eq_check(present_type, location.clone(), errors)?;
                        Ok(present_type.clone())
                    }
                    _ => {
                        let error = Error::ExpectOption {
                            given_type: option_type.clone(),
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
