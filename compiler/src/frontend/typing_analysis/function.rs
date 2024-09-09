//! LanGRust [Function] typing analysis module.

prelude! {
    frontend::TypeAnalysis,
    hir::Function,
}

impl TypeAnalysis for Function {
    fn typing(&mut self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        let Function {
            id,
            statements,
            returned,
            location,
            ..
        } = self;

        // type all statements
        statements
            .iter_mut()
            .map(|statement| {
                statement
                    .pattern
                    .typing(symbol_table, errors)?;
                statement.typing(symbol_table, errors)
            })
            .collect::<TRes<()>>()?;

        // type returned expression
        returned.typing(symbol_table, errors)?;

        // check returned type
        let expected_type = symbol_table.get_function_output_type(*id);
        returned
            .get_type()
            .unwrap()
            .eq_check(expected_type, location.clone(), errors)
    }
}
