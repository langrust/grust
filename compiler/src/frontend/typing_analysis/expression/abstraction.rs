prelude! {
    frontend::TypeAnalysis,
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the abstraction expression.
    pub fn typing_abstraction(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        match self {
            // the type of a typed abstraction is computed by adding inputs to
            // the context and typing the function body expression
            hir::expr::Kind::Abstraction {
                ref inputs,
                ref mut expression,
            } => {
                // type the abstracted expression with the local context
                expression.typing(symbol_table, errors)?;

                // compute abstraction type
                let input_types = inputs
                    .iter()
                    .map(|id| symbol_table.get_type(*id).clone())
                    .collect::<Vec<_>>();
                let abstraction_type =
                    Typ::function(input_types, expression.get_type().unwrap().clone());

                Ok(abstraction_type)
            }
            _ => unreachable!(),
        }
    }
}
