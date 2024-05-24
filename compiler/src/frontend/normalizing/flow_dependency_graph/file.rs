use crate::{hir::file::File, symbol_table::SymbolTable};

impl File {
    /// Generate dependency graph for every nodes/component.
    #[inline]
    pub fn generate_flows_dependency_graphs(&mut self, symbol_table: &SymbolTable) {
        self.interface.compute_dependencies(symbol_table)
    }
}
