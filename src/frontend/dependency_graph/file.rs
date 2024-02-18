use std::collections::HashMap;

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

        // creates nodes context: nodes dictionary
        let nodes_context = nodes
            .iter()
            .map(|node| (node.id.clone(), node.clone()))
            .collect::<HashMap<_, _>>();

        // every nodes complete their equations and contract dependency graphs
        nodes
            .iter()
            .map(|node| {
                node.add_contract_dependencies(&mut nodes_graphs);
                node.add_all_equations_dependencies(
                    symbol_table,
                    &nodes_context,
                    &mut nodes_processus_manager,
                    &mut nodes_reduced_processus_manager,
                    &mut nodes_graphs,
                    &mut nodes_reduced_graphs,
                    errors,
                )
            })
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // optional component completes its dependency graph
        component.as_ref().map_or(Ok(()), |component| {
            component.add_contract_dependencies(&mut nodes_graphs);
            component.add_all_equations_dependencies(
                symbol_table,
                &nodes_context,
                &mut nodes_processus_manager,
                &mut nodes_reduced_processus_manager,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                errors,
            )
        })?;

        Ok(())
    }
}
