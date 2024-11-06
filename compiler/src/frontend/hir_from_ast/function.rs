prelude! {
    ast::{ Function, stmt::Return },
}

impl<'a> HIRFromAST<hir::ctx::Simple<'a>> for Function {
    type HIR = hir::Function;

    // precondition: function and its inputs are already stored in symbol table
    // postcondition: construct HIR function and check identifiers good use
    fn hir_from_ast(self, ctxt: &mut hir::ctx::Simple<'a>) -> TRes<Self::HIR> {
        let Function {
            ident,
            output_type,
            contract,
            statements,
            ..
        } = self;
        let name = ident.to_string();
        let location = Location::default();
        let id = ctxt
            .syms
            .get_function_id(&name, false, location.clone(), ctxt.errors)?;

        // create local context with all signals
        ctxt.syms.local();
        ctxt.syms.restore_context(id);

        // insert function output type in symbol table
        let output_typing = output_type.hir_from_ast(&mut ctxt.add_loc(&location))?;
        if !contract.clauses.is_empty() {
            let _ = ctxt.syms.insert_function_result(
                output_typing.clone(),
                true,
                location.clone(),
                ctxt.errors,
            )?;
        }
        ctxt.syms.set_function_output_type(id, output_typing);

        let (statements, returned) = statements.into_iter().fold(
            (vec![], None),
            |(mut declarations, option_returned), statement| match statement {
                ast::Stmt::Declaration(declaration) => {
                    declarations.push(declaration.hir_from_ast(ctxt));
                    (declarations, option_returned)
                }
                ast::Stmt::Return(Return { expression, .. }) => {
                    assert!(option_returned.is_none());
                    (
                        declarations,
                        Some(expression.hir_from_ast(&mut ctxt.add_pat_loc(None, &location))),
                    )
                }
            },
        );
        let contract = contract.hir_from_ast(ctxt)?;

        ctxt.syms.global();

        Ok(hir::Function {
            id,
            contract,
            statements: statements.into_iter().collect::<TRes<Vec<_>>>()?,
            returned: returned.unwrap()?,
            location,
        })
    }
}
