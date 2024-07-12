prelude! {
    frontend::typing_analysis::TypeAnalysis,
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the tuple expression.
    pub fn typing_tuple(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Typ> {
        match self {
            // an tuple is composed of `n` elements of the different type `t_k` and
            // its type is `(t_1, ..., t_n)`
            hir::expr::Kind::Tuple { ref mut elements } => {
                debug_assert!(elements.len() >= 1);

                let elements_types = elements
                    .iter_mut()
                    .map(|element| {
                        element.typing(symbol_table, errors)?;
                        Ok(element.get_type().expect("should be typed").clone())
                    })
                    .collect::<TRes<Vec<Typ>>>()?;

                let tuple_type = Typ::tuple(elements_types);

                Ok(tuple_type)
            }
            _ => unreachable!(),
        }
    }
}
