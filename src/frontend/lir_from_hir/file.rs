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
            component,
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
        let mut nodes = nodes
            .into_iter()
            .flat_map(|node| {
                node.lir_from_hir(symbol_table)
                    .into_iter()
                    .map(Item::NodeFile)
            })
            .collect();
        let mut component = component.map_or(vec![], |component| {
            component
                .lir_from_hir(symbol_table)
                .into_iter()
                .map(Item::NodeFile)
                .collect()
        });

        let mut items = vec![];
        items.append(&mut typedefs);
        items.append(&mut functions);
        items.append(&mut nodes);
        items.append(&mut component);
        Project { items }
    }
}
