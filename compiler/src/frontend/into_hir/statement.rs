prelude! {
    ast::stmt::LetDecl,
}

impl IntoHir<hir::ctx::Simple<'_>> for LetDecl<ast::Expr> {
    type Hir = hir::Stmt<hir::Expr>;

    // precondition: NOTHING is in symbol table
    // postcondition: construct HIR statement and check identifiers good use
    fn into_hir(self, ctxt: &mut hir::ctx::Simple) -> TRes<Self::Hir> {
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
            expression.into_hir(&mut ctxt.add_pat_loc(Some(&typed_pattern), &location))?;
        let pattern = typed_pattern.into_hir(&mut ctxt.add_loc(&location))?;

        Ok(hir::Stmt {
            pattern,
            expression,
            location,
        })
    }
}
