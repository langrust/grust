prelude! {
    frontend::typing_analysis::TypeAnalysis, macro2::Span,
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the structure expression.
    pub fn typing_structure(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        match self {
            // the type of the structure is the corresponding structure type
            // if fields match their expected types
            hir::expr::Kind::Structure {
                ref id,
                ref mut fields,
            } => {
                // type each field and check their type
                fields
                    .iter_mut()
                    .map(|(id, expression)| {
                        expression.typing(symbol_table, errors)?;
                        let expression_type = expression.get_type().unwrap();
                        let expected_type = symbol_table.get_type(*id);
                        expression_type.eq_check(expected_type, location.clone(), errors)
                    })
                    .collect::<TRes<()>>()?;

                Ok(Typ::Structure {
                    name: syn::Ident::new(symbol_table.get_name(*id), Span::call_site()),
                    id: *id,
                })
            }
            _ => unreachable!(),
        }
    }
}
