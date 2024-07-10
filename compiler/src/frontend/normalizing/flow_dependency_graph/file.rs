prelude! { just
    hir::File,
}

impl File {
    /// Generate dependency graphs for the interface.
    #[inline]
    pub fn generate_flows_dependency_graphs(&mut self) {
        self.interface.generate_flows_dependency_graphs()
    }
}
