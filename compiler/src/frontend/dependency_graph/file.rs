prelude! {
    graph::*,
    hir::File,
}

use super::ctx;
use petgraph::algo::toposort;

impl File {
    /// Generate dependency graph for every nodes/component.
    pub fn generate_dependency_graphs(
        &mut self,
        symbol_table: &SymbolTable,
        errors: &mut Vec<Error>,
    ) -> TRes<()> {
        let File { components, .. } = self;

        // initialize dictionariy for reduced graphs
        let mut nodes_reduced_graphs = HashMap::new();

        // create graph of nodes
        let mut nodes_graph = DiGraphMap::new();
        components
            .iter()
            .for_each(|component| component.add_node_dependencies(&mut nodes_graph));

        // sort nodes according to their dependencies
        let sorted_nodes = toposort(&nodes_graph, None).map_err(|component| {
            let error = Error::NotCausalNode {
                node: symbol_table.get_name(component.node_id()).clone(),
                location: self.location.clone(),
            };
            errors.push(error);
            TerminationError
        })?;
        components.sort_by(|c1, c2| {
            let index1 = sorted_nodes
                .iter()
                .position(|id| *id == c1.get_id())
                .expect("should be in sorted list");
            let index2 = sorted_nodes
                .iter()
                .position(|id| *id == c2.get_id())
                .expect("should be in sorted list");

            Ord::cmp(&index2, &index1)
        });

        // ordered nodes complete their dependency graphs
        let mut ctx = ctx::Ctx::new(symbol_table, &mut nodes_reduced_graphs, errors);
        components
            .iter_mut()
            .map(|component| component.compute_dependencies(&mut ctx))
            .collect::<TRes<()>>()?;

        Ok(())
    }
}
