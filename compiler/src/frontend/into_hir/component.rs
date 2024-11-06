prelude! {}

impl IntoHir<hir::ctx::Simple<'_>> for ast::Component {
    type Hir = hir::Component;

    // precondition: node and its signals are already stored in symbol table
    // postcondition: construct HIR node and check identifiers good use
    fn into_hir(self, ctxt: &mut hir::ctx::Simple) -> TRes<Self::Hir> {
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
            .map(|equation| equation.into_hir(ctxt))
            .collect::<TRes<Vec<_>>>()?;
        let contract = contract.into_hir(ctxt)?;

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

impl IntoHir<hir::ctx::Simple<'_>> for ast::ComponentImport {
    type Hir = hir::Component;

    // precondition: node and its signals are already stored in symbol table
    // postcondition: construct HIR node
    fn into_hir(self, ctxt: &mut hir::ctx::Simple) -> TRes<Self::Hir> {
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
