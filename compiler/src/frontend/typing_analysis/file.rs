//! LanGRust [`File`] typing analysis module.

prelude! {
    frontend::typing_analysis::TypeAnalysis,
    hir::File,
}

impl TypeAnalysis for File {
    fn typing(&mut self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        let File {
            functions,
            nodes,
            interface,
            ..
        } = self;

        // typing nodes
        nodes
            .iter_mut()
            .map(|node| node.typing(symbol_table, errors))
            .collect::<TRes<()>>()?;

        // typing functions
        functions
            .iter_mut()
            .map(|function| function.typing(symbol_table, errors))
            .collect::<TRes<()>>()?;

        // typing interface
        interface
            .services
            .iter_mut()
            .map(|service| {
                service
                    .statements
                    .values_mut()
                    .map(|statement| statement.typing(symbol_table, errors))
                    .collect::<TRes<()>>()
            })
            .collect::<TRes<()>>()
    }
}
