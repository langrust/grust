prelude! {
    ast::Ast,
    hir::interface::Interface,
}

use super::HIRFromAST;

impl HIRFromAST for Ast {
    type HIR = hir::File;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        // initialize symbol table with builtin operators
        symbol_table.initialize();

        // store elements in symbol table
        self.store(symbol_table, errors)?;

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
                        components.push(component.hir_from_ast(symbol_table, errors))
                    }
                    ast::Item::Function(function) => {
                        functions.push(function.hir_from_ast(symbol_table, errors))
                    }
                    ast::Item::Typedef(typedef) => {
                        typedefs.push(typedef.hir_from_ast(symbol_table, errors))
                    }
                    ast::Item::Service(service) => {
                        services.push(service.hir_from_ast(symbol_table, errors))
                    }
                    ast::Item::Import(import) => imports.push(
                        import
                            .hir_from_ast(symbol_table, errors)
                            .map(|res| (symbol_table.get_fresh_id(), res)),
                    ),
                    ast::Item::Export(export) => exports.push(
                        export
                            .hir_from_ast(symbol_table, errors)
                            .map(|res| (symbol_table.get_fresh_id(), res)),
                    ),
                    ast::Item::ComponentImport(component) => {
                        components.push(component.hir_from_ast(symbol_table, errors))
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
