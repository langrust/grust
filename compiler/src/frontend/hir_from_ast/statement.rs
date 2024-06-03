prelude! {
    ast::stmt::LetDecl,
}

use super::HIRFromAST;

impl HIRFromAST for LetDecl<ast::Expr> {
    type HIR = hir::Stmt<hir::Expr>;

    // precondition: NOTHING is in symbol table
    // postcondition: construct HIR statement and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        let LetDecl {
            typed_pattern,
            expression,
            ..
        } = self;
        let location = Location::default();
        typed_pattern.store(true, symbol_table, errors)?;
        let pattern = typed_pattern.hir_from_ast(symbol_table, errors)?;

        Ok(hir::Stmt {
            pattern,
            expression: expression.hir_from_ast(symbol_table, errors)?,
            location,
        })
    }
}
