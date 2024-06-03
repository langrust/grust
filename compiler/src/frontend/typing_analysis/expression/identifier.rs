prelude! {
    frontend::TypeAnalysis,
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the call expression.
    pub fn typing_identifier(&mut self, symbol_table: &mut SymbolTable) -> TRes<Typ> {
        match self {
            // the type of a call expression in the type of the called element in the context
            hir::expr::Kind::Identifier { ref id } => {
                let typing = symbol_table.get_type(*id);
                Ok(typing.clone())
            }
            _ => unreachable!(),
        }
    }
}
