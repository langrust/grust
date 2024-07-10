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

        let (typedefs, functions, nodes, services) = items.into_iter().fold(
            (vec![], vec![], vec![], vec![]),
            |(mut typedefs, mut functions, mut nodes, mut services), item| match item {
                ast::Item::Component(component) => {
                    nodes.push(component.hir_from_ast(symbol_table, errors));
                    (typedefs, functions, nodes, services)
                }
                ast::Item::Function(function) => {
                    functions.push(function.hir_from_ast(symbol_table, errors));
                    (typedefs, functions, nodes, services)
                }
                ast::Item::Typedef(typedef) => {
                    typedefs.push(typedef.hir_from_ast(symbol_table, errors));
                    (typedefs, functions, nodes, services)
                }
                ast::Item::Service(service) => {
                    services.push(service.hir_from_ast(symbol_table, errors));
                    (typedefs, functions, nodes, services)
                }
            },
        );

        let interface = Interface {
            services: services.into_iter().collect::<TRes<Vec<_>>>()?,
        };

        Ok(hir::File {
            typedefs: typedefs.into_iter().collect::<TRes<Vec<_>>>()?,
            functions: functions.into_iter().collect::<TRes<Vec<_>>>()?,
            nodes: nodes.into_iter().collect::<TRes<Vec<_>>>()?,
            interface,
            location: Location::default(),
        })
    }
}
