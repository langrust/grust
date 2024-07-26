prelude! {}

use super::HIRFromAST;

impl HIRFromAST for ast::Component {
    type HIR = hir::Component;

    // precondition: node and its signals are already stored in symbol table
    // postcondition: construct HIR node and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        let ast::Component {
            ident,
            contract,
            equations,
            ..
        } = self;
        let name = ident.to_string();
        let location = Location::default();
        let id = symbol_table.get_node_id(&name, false, location.clone(), errors)?;

        // create local context with all signals
        symbol_table.local();
        symbol_table.restore_context(id);
        symbol_table.enter_in_node(id);

        let statements = equations
            .into_iter()
            .map(|equation| equation.hir_from_ast(symbol_table, errors))
            .collect::<TRes<Vec<_>>>()?;
        let contract = contract.hir_from_ast(symbol_table, errors)?;

        symbol_table.leave_node();
        symbol_table.global();

        Ok(hir::Component::Definition(hir::ComponentDefinition {
            id,
            statements,
            contract,
            location,
            graph: graph::DiGraphMap::new(),
            memory: hir::Memory::new(),
        }))
    }
}

impl HIRFromAST for ast::ComponentImport {
    type HIR = hir::Component;

    // precondition: node and its signals are already stored in symbol table
    // postcondition: construct HIR node
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<Self::HIR> {
        let ast::ComponentImport { path, .. } = self;

        let last = path.clone().segments.pop().unwrap().into_value();
        let name = last.ident.to_string();
        assert!(last.arguments.is_none());

        let location = Location::default();
        let id = symbol_table.get_node_id(&name, false, location.clone(), errors)?;

        Ok(hir::Component::Import(hir::ComponentImport {
            id,
            path,
            location,
            graph: graph::DiGraphMap::new(),
        }))
    }
}
