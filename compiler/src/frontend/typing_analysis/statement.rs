//! LanGRust [Stmt] typing analysis module.

prelude! {
    frontend::typing_analysis::TypeAnalysis,
    hir::Stmt,
}

impl<E> TypeAnalysis for Stmt<E>
where
    E: TypeAnalysis,
{
    // precondition: identifiers associated with statement is already typed
    // postcondition: expression associated with statement is typed and checked
    fn typing(&mut self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        let Stmt {
            pattern,
            expression,
            location,
        } = self;

        pattern.typing(symbol_table, errors)?;
        let expected_type = pattern.typing.as_ref().unwrap();

        expression.typing(symbol_table, errors)?;
        let expression_type = expression.get_type().unwrap();

        expression_type.eq_check(expected_type, location.clone(), errors)?;

        Ok(())
    }
}
