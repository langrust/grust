prelude! {
    frontend::TypeAnalysis,
}

impl<E> hir::expr::Kind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Typ] to the constant expression.
    pub fn typing_constant(&mut self) -> TRes<Typ> {
        match self {
            // typing a constant expression consist of getting the type of the constant
            hir::expr::Kind::Constant { ref constant } => {
                let constant_type = constant.get_type();
                Ok(constant_type)
            }
            _ => unreachable!(),
        }
    }
}
