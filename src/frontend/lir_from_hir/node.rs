use itertools::Itertools;

use crate::{hir::node::Node, lir::item::node_file::NodeFile, symbol_table::SymbolTable};

use super::LIRFromHIR;

impl LIRFromHIR for Node {
    type LIR = Vec<NodeFile>;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        self.unitary_nodes
            .into_iter()
            .sorted_by_key(|(id, _)| *id)
            .map(|(_, unitary_node)| unitary_node.lir_from_hir(symbol_table))
            .collect()
    }
}
