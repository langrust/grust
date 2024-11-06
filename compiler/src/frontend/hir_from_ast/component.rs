prelude! {}

impl<'a> HIRFromAST<hir::ctx::Simple<'a>> for ast::Component {
    type HIR = hir::Component;

    // precondition: node and its signals are already stored in symbol table
    // postcondition: construct HIR node and check identifiers good use
    fn hir_from_ast(self, ctxt: &mut hir::ctx::Simple<'a>) -> TRes<Self::HIR> {
        let ast::Component {
            ident,
            contract,
            equations,
            ..
        } = self;
        let name = ident.to_string();
        let location = Location::default();
        let id = ctxt
            .syms
            .get_node_id(&name, false, location.clone(), ctxt.errors)?;

        // create local context with all signals
        ctxt.syms.local();
        ctxt.syms.restore_context(id);

        let statements = equations
            .into_iter()
            .map(|equation| equation.hir_from_ast(ctxt))
            .collect::<TRes<Vec<_>>>()?;
        let contract = contract.hir_from_ast(ctxt)?;

        ctxt.syms.global();

        Ok(hir::Component::Definition(hir::ComponentDefinition {
            id,
            statements,
            contract,
            location,
            graph: graph::DiGraphMap::new(),
            reduced_graph: graph::DiGraphMap::new(),
            memory: hir::Memory::new(),
        }))
    }
}

impl<'a> HIRFromAST<hir::ctx::Simple<'a>> for ast::ComponentImport {
    type HIR = hir::Component;

    // precondition: node and its signals are already stored in symbol table
    // postcondition: construct HIR node
    fn hir_from_ast(self, ctxt: &mut hir::ctx::Simple<'a>) -> TRes<Self::HIR> {
        let ast::ComponentImport { path, .. } = self;

        let last = path.clone().segments.pop().unwrap().into_value();
        let name = last.ident.to_string();
        assert!(last.arguments.is_none());

        let location = Location::default();
        let id = ctxt
            .syms
            .get_node_id(&name, false, location.clone(), ctxt.errors)?;

        Ok(hir::Component::Import(hir::ComponentImport {
            id,
            path,
            location,
            graph: graph::DiGraphMap::new(),
        }))
    }
}
