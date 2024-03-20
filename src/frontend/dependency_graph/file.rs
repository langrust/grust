use std::collections::HashMap;

use itertools::Itertools;
use petgraph::algo::toposort;
use petgraph::graphmap::DiGraphMap;

use crate::error::{Error, TerminationError};
use crate::hir::file::File;
use crate::symbol_table::SymbolTable;

impl File {
    /// Generate dependency graph for every nodes/component.
    pub fn generate_dependency_graphs(
        &self,
        symbol_table: &SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let File {
            nodes, component, ..
        } = self;

        // initialize dictionaries for graphs
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut nodes_processus_manager = HashMap::new();
        let mut nodes_reduced_processus_manager = HashMap::new();

        // initialize every nodes' graphs
        nodes
            .iter()
            .map(|node| {
                let graph = node.create_initialized_graph(symbol_table);
                nodes_graphs.insert(node.id.clone(), graph.clone());
                nodes_reduced_graphs.insert(node.id.clone(), graph);
                let processus_manager = node.create_initialized_processus_manager(symbol_table);
                nodes_processus_manager.insert(node.id.clone(), processus_manager.clone());
                nodes_reduced_processus_manager.insert(node.id.clone(), processus_manager);
                Ok(())
            })
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // optional component's graph initialization
        component.as_ref().map_or(Ok(()), |component| {
            let graph = component.create_initialized_graph(symbol_table);
            nodes_graphs.insert(component.id.clone(), graph.clone());
            nodes_reduced_graphs.insert(component.id.clone(), graph);
            let processus_manager = component.create_initialized_processus_manager(symbol_table);
            nodes_processus_manager.insert(component.id.clone(), processus_manager.clone());
            nodes_reduced_processus_manager.insert(component.id.clone(), processus_manager);
            Ok(())
        })?;

        // create graph of nodes
        let mut nodes_graph = DiGraphMap::new();
        nodes
            .iter()
            .for_each(|node| node.add_node_dependencies(&mut nodes_graph));
        if let Some(component) = component {
            component.add_node_dependencies(&mut nodes_graph)
        }

        // sort nodes according to their dependencies
        let sorted_nodes = toposort(&nodes_graph, None).map_err(|node| {
            let error = Error::NotCausalNode {
                node: symbol_table.get_name(node.node_id()).clone(),
                location: self.location.clone(),
            };
            errors.push(error);
            TerminationError
        })?;

        // nodes complete their contract dependency graphs
        nodes
            .iter()
            .for_each(|node| node.add_contract_dependencies(&mut nodes_graphs));

        // optional component completes its contract dependency graph
        if let Some(component) = component {
            component.add_contract_dependencies(&mut nodes_graphs)
        }

        // ordered nodes complete their equations dependency graphs
        nodes
            .iter()
            .sorted_by(|n1, n2| {
                let index1 = sorted_nodes
                    .iter()
                    .position(|id| *id == n1.id)
                    .expect(&format!(
                        "node '{}' should be in sorted nodes",
                        symbol_table.get_name(n1.id)
                    ));
                let index2 = sorted_nodes
                    .iter()
                    .position(|id| *id == n2.id)
                    .expect(&format!(
                        "node '{}' should be in sorted nodes",
                        symbol_table.get_name(n2.id)
                    ));
                Ord::cmp(&index2, &index1)
            })
            .map(|node| {
                node.add_equations_dependencies(
                    symbol_table,
                    &mut nodes_processus_manager,
                    &mut nodes_graphs,
                    &mut nodes_reduced_graphs,
                    errors,
                )
            })
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // optional component completes its equations dependency graph
        component.as_ref().map_or(Ok(()), |component| {
            component.add_equations_dependencies(
                symbol_table,
                &mut nodes_processus_manager,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                errors,
            )
        })?;

        Ok(())
    }
}
