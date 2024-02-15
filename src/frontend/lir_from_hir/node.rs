use itertools::Itertools;

use crate::{hir::node::Node, lir::item::node_file::NodeFile, symbol_table::SymbolTable};

use super::unitary_node::lir_from_hir as unitary_node_lir_from_hir;

/// Transform HIR node into LIR node files.
pub fn lir_from_hir(node: Node, symbol_table: &SymbolTable) -> Vec<NodeFile> {
    node.unitary_nodes
        .into_iter()
        .sorted_by_key(|(id, _)| id.clone())
        .map(|(_, unitary_node)| unitary_node_lir_from_hir(unitary_node, symbol_table))
        .collect()
}
