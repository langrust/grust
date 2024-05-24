use crate::{
    hir::file::File,
    lir::{item::Item, project::Project},
    symbol_table::SymbolTable,
};

use super::LIRFromHIR;

impl LIRFromHIR for File {
    type LIR = Project;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        let File {
            typedefs,
            functions,
            nodes,
            interface,
            ..
        } = self;

        let mut typedefs = typedefs
            .into_iter()
            .map(|typedef| typedef.lir_from_hir(symbol_table))
            .collect();
        let mut functions = functions
            .into_iter()
            .map(|function| function.lir_from_hir(symbol_table))
            .map(Item::Function)
            .collect();
        let mut state_machines = nodes
            .into_iter()
            .map(|node| node.lir_from_hir(symbol_table))
            .map(Item::StateMachine)
            .collect();
        let execution_machine = interface.lir_from_hir(symbol_table);

        let mut items = vec![];
        items.append(&mut typedefs);
        items.append(&mut functions);
        items.append(&mut state_machines);
        items.push(Item::ExecutionMachine(execution_machine));

        Project { items }
    }
}
