//! LanGRust [Component] typing analysis module.

prelude! {
    frontend::TypeAnalysis,
    hir::{Component, ComponentDefinition},
}

impl TypeAnalysis for Component {
    fn typing(&mut self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        if let Component::Definition(comp_def) = self {
            comp_def.typing(symbol_table, errors)
        } else {
            Ok(())
        }
    }
}

impl TypeAnalysis for ComponentDefinition {
    fn typing(&mut self, symbol_table: &mut SymbolTable, errors: &mut Vec<Error>) -> TRes<()> {
        let ComponentDefinition {
            statements,
            contract,
            ..
        } = self;

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
            .collect::<TRes<()>>()?;

        // type contract
        contract.typing(symbol_table, errors)
    }
}
