prelude! {
    frontend::TypeAnalysis,
    hir::Node,
}

impl TypeAnalysis for Node {
    fn typing(&mut self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        let Node { statements, .. } = self;

        // set types for every pattern
        statements
            .iter_mut()
            .map(|statement| {
                statement
                    .pattern
                    .construct_statement_type(symbol_table, errors)
            })
            .collect::<TRes<()>>()?;

        // type all equations
        statements
            .iter_mut()
            .map(|statement| statement.typing(symbol_table, errors))
            .collect::<TRes<()>>()
    }
}
