prelude! {
    ast::pattern::Pattern,
}

use super::HIRFromAST;

impl HIRFromAST for Pattern {
    type HIR = hir::Pattern;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        let location = Location::default();
        let loc = &location;

        let kind = match self {
            Pattern::Constant(constant) => hir::pattern::Kind::Constant { constant },
            Pattern::Identifier(name) => {
                let id = symbol_table.get_identifier_id(&name, false, location.clone(), errors)?;
                hir::pattern::Kind::Identifier { id }
            }
            Pattern::Typed(pattern) => pattern.hir_from_ast(loc, symbol_table, errors)?,
            Pattern::Structure(pattern) => pattern.hir_from_ast(loc, symbol_table, errors)?,
            Pattern::Enumeration(pattern) => pattern.hir_from_ast(loc, symbol_table, errors)?,
            Pattern::Tuple(pattern) => pattern.hir_from_ast(loc, symbol_table, errors)?,
            // Pattern::None => hir::pattern::Kind::None,
            Pattern::Default => hir::pattern::Kind::Default,
        };

        Ok(hir::Pattern {
            kind,
            typing: None,
            location,
        })
    }
}
