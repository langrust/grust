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

        let mut typedefs = typedefs
            .into_iter()
            .map(|typedef| typedef.lir_from_hir(&symbol_table))
            .collect();
        let mut functions = functions
            .into_iter()
            .map(|function| function.lir_from_hir(&symbol_table))
            .map(Item::Function)
            .collect();
        let mut state_machines = nodes
            .into_iter()
            .map(|node| node.lir_from_hir(&symbol_table))
            .map(Item::StateMachine)
            .collect();
        let execution_machine = interface.lir_from_hir(&mut symbol_table);

        let mut items = vec![];
        items.append(&mut typedefs);
        items.append(&mut functions);
        items.append(&mut state_machines);
        items.push(Item::ExecutionMachine(execution_machine));

        Project { items }
    }
}
