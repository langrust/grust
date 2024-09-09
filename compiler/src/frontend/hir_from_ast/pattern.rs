prelude! {}

use super::HIRFromAST;

impl HIRFromAST for ast::expr::Pattern {
    type HIR = hir::Pattern;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        let location = Location::default();
        let loc = &location;

        let kind = match self {
            ast::expr::Pattern::Constant(constant) => hir::pattern::Kind::Constant { constant },
            ast::expr::Pattern::Identifier(name) => {
                let id = symbol_table.get_identifier_id(&name, false, location.clone(), errors)?;
                hir::pattern::Kind::Identifier { id }
            }
            ast::expr::Pattern::Structure(pattern) => {
                pattern.hir_from_ast(loc, symbol_table, errors)?
            }
            ast::expr::Pattern::Enumeration(pattern) => {
                pattern.hir_from_ast(loc, symbol_table, errors)?
            }
            ast::expr::Pattern::Tuple(pattern) => {
                pattern.hir_from_ast(loc, symbol_table, errors)?
            }
            // Pattern::None => hir::pattern::Kind::None,
            ast::expr::Pattern::Default => hir::pattern::Kind::Default,
        };

        Ok(hir::Pattern {
            kind,
            typing: None,
            location,
        })
    }
}

impl HIRFromAST for ast::stmt::Pattern {
    type HIR = hir::stmt::Pattern;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        let location = Location::default();
        let loc = &location;

        let kind = match self {
            ast::stmt::Pattern::Identifier(ident) => {
                let id = symbol_table.get_identifier_id(
                    &ident.to_string(),
                    false,
                    location.clone(),
                    errors,
                )?;
                hir::stmt::Kind::Identifier { id }
            }
            ast::stmt::Pattern::Typed(pattern) => {
                pattern.hir_from_ast(loc, symbol_table, errors)?
            }
            ast::stmt::Pattern::Tuple(pattern) => {
                pattern.hir_from_ast(loc, symbol_table, errors)?
            }
        };

        Ok(hir::stmt::Pattern {
            kind,
            typing: None,
            location,
        })
    }
}
