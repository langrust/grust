prelude! {
    frontend::typing_analysis::TypeAnalysis, macro2::Span
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the enumeration expression.
    pub fn typing_enumeration(&mut self, symbol_table: &mut SymbolTable) -> TRes<Typ> {
        match self {
            // the type of the enumeration is the corresponding enumeration type
            hir::expr::Kind::Enumeration { ref enum_id, .. } => {
                // type each field and check their type
                Ok(Typ::Enumeration {
                    name: syn::Ident::new(symbol_table.get_name(*enum_id), Span::call_site()),
                    id: *enum_id,
                })
            }
            _ => unreachable!(),
        }
    }
}
