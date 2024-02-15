use crate::{
    hir::file::File,
    lir::{item::Item, project::Project},
    symbol_table::SymbolTable,
};

use super::{
    function::lir_from_hir as function_lir_from_hir, node::lir_from_hir as node_lir_from_hir,
    typedef::lir_from_hir as typedef_lir_from_hir,
};

/// Transform HIR file into LIR project.
pub fn lir_from_hir(file: File, symbol_table: &SymbolTable) -> Project {
    let File {
        typedefs,
        functions,
        nodes,
        component,
        ..
    } = file;

    let mut typedefs = typedefs
        .into_iter()
        .map(|typedef| typedef_lir_from_hir(typedef, symbol_table))
        .collect();
    let mut functions = functions
        .into_iter()
        .map(|function| function_lir_from_hir(function, symbol_table))
        .map(Item::Function)
        .collect();
    let mut nodes = nodes
        .into_iter()
        .flat_map(|node| {
            node_lir_from_hir(node, symbol_table)
                .into_iter()
                .map(Item::NodeFile)
        })
        .collect();
    let mut component = component.map_or(vec![], |component| {
        node_lir_from_hir(component, symbol_table)
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
