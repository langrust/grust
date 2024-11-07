prelude! {
    hir::File, lir::{Item, Project},
}

impl IntoLir<SymbolTable> for File {
    type Lir = Project;

    fn into_lir(self, mut symbol_table: SymbolTable) -> Project {
        let mut items = vec![];

        let typedefs = self
            .typedefs
            .into_iter()
            .map(|typedef| typedef.into_lir(&symbol_table));
        items.extend(typedefs);

        let functions = self
            .functions
            .into_iter()
            .map(|function| function.into_lir(&symbol_table))
            .map(Item::Function);
        items.extend(functions);

        let state_machines = self
            .components
            .into_iter()
            .map(|component| component.into_lir(&symbol_table));
        items.extend(state_machines);

        let execution_machines = self.interface.into_lir(&mut symbol_table);
        items.push(Item::ExecutionMachine(execution_machines));

        Project { items }
    }
}
