prelude! {
    ast::stmt::LetDecl,
}

use super::{HIRFromAST, SimpleCtxt};

impl<'a> HIRFromAST<SimpleCtxt<'a>> for LetDecl<ast::Expr> {
    type HIR = hir::Stmt<hir::Expr>;

    // precondition: NOTHING is in symbol table
    // postcondition: construct HIR statement and check identifiers good use
    fn hir_from_ast(self, ctxt: &mut SimpleCtxt<'a>) -> TRes<Self::HIR> {
        let LetDecl {
            typed_pattern,
            expression,
            ..
        } = self;
        let location = Location::default();

        // stmts should be ordered in functions
        // then patterns are stored in order
        typed_pattern.store(true, ctxt.syms, ctxt.errors)?;
        let expression =
            expression.hir_from_ast(&mut ctxt.add_pat_loc(Some(&typed_pattern), &location))?;
        let pattern = typed_pattern.hir_from_ast(&mut ctxt.add_loc(&location))?;

        Ok(hir::Stmt {
            pattern,
            expression,
            location,
        })
    }
}
