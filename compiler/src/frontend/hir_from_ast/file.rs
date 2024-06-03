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

        let (typedefs, functions, nodes, flow_statements) = items.into_iter().fold(
            (vec![], vec![], vec![], vec![]),
            |(mut typedefs, mut functions, mut nodes, mut flow_statements), item| match item {
                ast::Item::Component(component) => {
                    nodes.push(component.hir_from_ast(symbol_table, errors));
                    (typedefs, functions, nodes, flow_statements)
                }
                ast::Item::Function(function) => {
                    functions.push(function.hir_from_ast(symbol_table, errors));
                    (typedefs, functions, nodes, flow_statements)
                }
                ast::Item::Typedef(typedef) => {
                    typedefs.push(typedef.hir_from_ast(symbol_table, errors));
                    (typedefs, functions, nodes, flow_statements)
                }
                ast::Item::FlowStatement(flow_statement) => {
                    flow_statements.push(flow_statement.hir_from_ast(symbol_table, errors));
                    (typedefs, functions, nodes, flow_statements)
                }
            },
        );

        let interface = Interface {
            statements: flow_statements.into_iter().collect::<TRes<Vec<_>>>()?,
            graph: Default::default(),
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
