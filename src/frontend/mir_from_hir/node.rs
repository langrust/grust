use crate::{hir::node::Node, mir::item::node_file::NodeFile};

use super::unitary_node::mir_from_hir as unitary_node_mir_from_hir;

/// Transform HIR node into MIR node files.
pub fn mir_from_hir(node: Node) -> Vec<NodeFile> {
    node.unitary_nodes
        .into_values()
        .map(unitary_node_mir_from_hir)
        .collect()
}
