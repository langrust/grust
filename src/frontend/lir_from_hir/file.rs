use crate::{
    hir::file::File,
    lir::{item::Item, project::Project},
};

use super::{
    function::lir_from_hir as function_lir_from_hir, node::lir_from_hir as node_lir_from_hir,
    typedef::lir_from_hir as typedef_lir_from_hir,
};

/// Transform HIR file into LIR project.
pub fn lir_from_hir(file: File) -> Project {
    let File {
        typedefs,
        functions,
        nodes,
        component,
        ..
    } = file;

    let mut typedefs = typedefs.into_iter().map(typedef_lir_from_hir).collect();
    let mut functions = functions
        .into_iter()
        .map(function_lir_from_hir)
        .map(Item::Function)
        .collect();
    let mut nodes = nodes
        .into_iter()
        .flat_map(|node| node_lir_from_hir(node).into_iter().map(Item::NodeFile))
        .collect();
    let mut component = component.map_or(vec![], |component| {
        node_lir_from_hir(component)
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
