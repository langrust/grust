prelude! {
    ast::Ast,
    hir::interface::Interface,
}

impl<'a> HIRFromAST<hir::ctx::Simple<'a>> for Ast {
    type HIR = hir::File;

    fn hir_from_ast(self, ctxt: &mut hir::ctx::Simple<'a>) -> TRes<Self::HIR> {
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
                    ast::Item::Component(component) => {
                        components.push(component.hir_from_ast(ctxt))
                    }
                    ast::Item::Function(function) => functions.push(function.hir_from_ast(ctxt)),
                    ast::Item::Typedef(typedef) => typedefs.push(typedef.hir_from_ast(ctxt)),
                    ast::Item::Service(service) => services.push(service.hir_from_ast(ctxt)),
                    ast::Item::Import(import) => imports.push(
                        import
                            .hir_from_ast(ctxt)
                            .map(|res| (ctxt.syms.get_fresh_id(), res)),
                    ),
                    ast::Item::Export(export) => exports.push(
                        export
                            .hir_from_ast(ctxt)
                            .map(|res| (ctxt.syms.get_fresh_id(), res)),
                    ),
                    ast::Item::ComponentImport(component) => {
                        components.push(component.hir_from_ast(ctxt))
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
