prelude! {
    hir::File, lir::{Item, Project},
}

use super::LIRFromHIR;

impl File {
    pub fn lir_from_hir(self, mut symbol_table: SymbolTable) -> Project {
        let File {
            typedefs,
            functions,
            nodes,
            interface,
            ..
        } = self;

        let mut items = vec![];

        let typedefs = typedefs
            .into_iter()
            .map(|typedef| typedef.lir_from_hir(&symbol_table));
        items.extend(typedefs);

        let functions = functions
            .into_iter()
            .map(|function| function.lir_from_hir(&symbol_table))
            .map(Item::Function);
        items.extend(functions);

        let state_machines = nodes
            .into_iter()
            .map(|node| node.lir_from_hir(&symbol_table))
            .map(Item::StateMachine);
        items.extend(state_machines);

        let execution_machines = interface.lir_from_hir(&mut symbol_table);
        items.push(Item::ExecutionMachine(execution_machines));

        Project { items }
    }
}
