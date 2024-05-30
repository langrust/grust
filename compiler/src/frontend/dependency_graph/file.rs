use petgraph::algo::toposort;
use petgraph::graphmap::DiGraphMap;

prelude! {
    error::{Error, TerminationError},
    hir::file::File,
    symbol_table::SymbolTable,

}

impl File {
    /// Generate dependency graph for every nodes/component.
    pub fn generate_dependency_graphs(
        &mut self,
        symbol_table: &SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let File { nodes, .. } = self;

        // initialize dictionariy for reduced graphs
        let mut nodes_reduced_graphs = HashMap::new();

        // create graph of nodes
        let mut nodes_graph = DiGraphMap::new();
        nodes
            .iter()
            .for_each(|node| node.add_node_dependencies(&mut nodes_graph));

        // sort nodes according to their dependencies
        let sorted_nodes = toposort(&nodes_graph, None).map_err(|node| {
            let error = Error::NotCausalNode {
                node: symbol_table.get_name(node.node_id()).clone(),
                location: self.location.clone(),
            };
            errors.push(error);
            TerminationError
        })?;
        nodes.sort_by(|n1, n2| {
            let index1 = sorted_nodes
                .iter()
                .position(|id| *id == n1.id)
                .expect("should be in sorted list");
            let index2 = sorted_nodes
                .iter()
                .position(|id| *id == n2.id)
                .expect("should be in sorted list");

            Ord::cmp(&index2, &index1)
        });

        // ordered nodes complete their dependency graphs
        nodes
            .iter_mut()
            .map(|node| node.compute_dependencies(symbol_table, &mut nodes_reduced_graphs, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        Ok(())
    }
}
