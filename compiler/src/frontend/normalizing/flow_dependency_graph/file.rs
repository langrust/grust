prelude! { just
    hir::File,
}

impl File {
    /// Generate dependency graph for every nodes/component.
    #[inline]
    pub fn generate_flows_dependency_graphs(&mut self) {
        self.interface.compute_dependencies()
    }
}
