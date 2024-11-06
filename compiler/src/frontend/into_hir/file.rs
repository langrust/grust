prelude! {
    ast::Ast,
    hir::interface::Interface,
}

impl IntoHir<hir::ctx::Simple<'_>> for Ast {
    type Hir = hir::File;

    fn into_hir(self, ctxt: &mut hir::ctx::Simple) -> TRes<Self::Hir> {
        // initialize symbol table with builtin operators
        ctxt.syms.initialize();

        // store elements in symbol table
        self.store(ctxt.syms, ctxt.errors)?;

        let Ast { items } = self;

        let (typedefs, functions, components, imports, exports, services) = items.into_iter().fold(
            (vec![], vec![], vec![], vec![], vec![], vec![]),
            |(
                mut typedefs,
                mut functions,
                mut components,
                mut imports,
                mut exports,
                mut services,
            ),
             item| {
                match item {
                    ast::Item::Component(component) => components.push(component.into_hir(ctxt)),
                    ast::Item::Function(function) => functions.push(function.into_hir(ctxt)),
                    ast::Item::Typedef(typedef) => typedefs.push(typedef.into_hir(ctxt)),
                    ast::Item::Service(service) => services.push(service.into_hir(ctxt)),
                    ast::Item::Import(import) => imports.push(
                        import
                            .into_hir(ctxt)
                            .map(|res| (ctxt.syms.get_fresh_id(), res)),
                    ),
                    ast::Item::Export(export) => exports.push(
                        export
                            .into_hir(ctxt)
                            .map(|res| (ctxt.syms.get_fresh_id(), res)),
                    ),
                    ast::Item::ComponentImport(component) => {
                        components.push(component.into_hir(ctxt))
                    }
                }
                (typedefs, functions, components, imports, exports, services)
            },
        );

        let interface = Interface {
            services: services.into_iter().collect::<TRes<Vec<_>>>()?,
            imports: imports.into_iter().collect::<TRes<_>>()?,
            exports: exports.into_iter().collect::<TRes<_>>()?,
        };

        Ok(hir::File {
            typedefs: typedefs.into_iter().collect::<TRes<Vec<_>>>()?,
            functions: functions.into_iter().collect::<TRes<Vec<_>>>()?,
            components: components.into_iter().collect::<TRes<Vec<_>>>()?,
            interface,
            location: Location::default(),
        })
    }
}
