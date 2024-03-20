use itertools::Itertools;
use petgraph::graphmap::DiGraphMap;
use std::collections::HashMap;

use crate::common::color::Color;
use crate::common::label::Label;
use crate::error::{Error, TerminationError};
use crate::hir::node::Node;
use crate::symbol_table::SymbolTable;

use super::add_edge;

impl Node {
    /// Create an initialized graph from a node.
    ///
    /// The created graph has every node's signals as vertices.
    /// But no edges are added.
    pub fn create_initialized_graph(&self, symbol_table: &SymbolTable) -> DiGraphMap<usize, Label> {
        // create an empty graph
        let mut graph = DiGraphMap::new();

        // add input signals as vertices
        for input in symbol_table.get_node_inputs(self.id) {
            graph.add_node(*input);
        }

        // add other signals as vertices
        for signal in self.unscheduled_equations.keys().sorted() {
            graph.add_node(*signal);
        }

        // return graph
        graph
    }

    /// Create an initialized processus manager from a node.
    pub fn create_initialized_processus_manager(
        &self,
        symbol_table: &SymbolTable,
    ) -> HashMap<usize, Color> {
        // create an empty hash
        let mut hash = HashMap::new();

        // add input signals with white color (unprocessed)
        for input in symbol_table.get_node_inputs(self.id) {
            hash.insert(*input, Color::White);
        }

        // add other signals with white color (unprocessed)
        for signal in self.unscheduled_equations.keys() {
            hash.insert(*signal, Color::White);
        }

        // return hash
        hash
    }

    /// Store nodes applications as dependencies.
    pub fn add_node_dependencies(&self, graph: &mut DiGraphMap<usize, ()>) {
        self.unscheduled_equations.values().for_each(|statement| {
            statement
                .expression
                .get_called_nodes()
                .into_iter()
                .for_each(|id| {
                    graph.add_edge(self.id, id, ());
                })
        });
    }

    /// Complete dependency graph of the node's equations.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(i: int) { // i depends on nothing
    ///     out o: int = x; // depends on x
    ///     x: int = i;     // depends on i
    /// }
    /// ```
    pub fn add_all_equations_dependencies(
        &self,
        symbol_table: &SymbolTable,
        nodes_context: &HashMap<usize, Node>,
        nodes_processus_manager: &mut HashMap<usize, HashMap<usize, Color>>,
        nodes_reduced_processus_manager: &mut HashMap<usize, HashMap<usize, Color>>,
        nodes_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Node {
            unscheduled_equations,
            graph,
            ..
        } = self;

        // add local and output signals dependencies
        unscheduled_equations
            .keys()
            .sorted()
            .map(|signal| {
                self.add_signal_dependencies(
                    *signal,
                    symbol_table,
                    nodes_context,
                    nodes_processus_manager,
                    nodes_reduced_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )
            })
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // add input signals dependencies
        symbol_table
            .get_node_inputs(self.id)
            .iter()
            .map(|signal| {
                self.add_signal_dependencies(
                    *signal,
                    symbol_table,
                    nodes_context,
                    nodes_processus_manager,
                    nodes_reduced_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )
            })
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // set node's graph
        graph
            .set(nodes_graphs.get(&self.id).unwrap().clone())
            .expect("should be the first time");

        Ok(())
    }

    /// Add direct dependencies of a signal.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(i: int) {
    ///     out o: int = x; // depends on x
    ///     x: int = i;     // depends on i
    /// }
    /// ```
    pub fn add_signal_dependencies(
        &self,
        signal: usize,
        symbol_table: &SymbolTable,
        nodes_context: &HashMap<usize, Node>,
        nodes_processus_manager: &mut HashMap<usize, HashMap<usize, Color>>,
        nodes_reduced_processus_manager: &mut HashMap<usize, HashMap<usize, Color>>,
        nodes_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Node {
            id: node,
            unscheduled_equations,
            location,
            ..
        } = self;

        // get node's processus manager
        let processus_manager = nodes_processus_manager.get_mut(node).unwrap();
        // get signal's color
        let color = processus_manager
            .get_mut(&signal)
            .expect("signal should be in processing manager");

        match color {
            // if vertex unprocessed
            Color::White => {
                // update status: processing
                *color = Color::Grey;

                unscheduled_equations
                    .get(&signal)
                    .map_or(Ok(()), |equation| {
                        // retrieve expression
                        let expression = &equation.expression;

                        // compute and get dependencies
                        expression.compute_dependencies(
                            symbol_table,
                            nodes_context,
                            nodes_processus_manager,
                            nodes_reduced_processus_manager,
                            nodes_graphs,
                            nodes_reduced_graphs,
                            errors,
                        )?;

                        let graph = nodes_graphs.get_mut(node).unwrap();
                        // add dependencies as graph's edges:
                        // s = e depends on s' <=> s -> s'
                        expression
                            .get_dependencies()
                            .iter()
                            .for_each(|(id, label)| {
                                // if there was another edge, keep the most important label
                                add_edge(graph, signal, *id, label.clone())
                            });

                        Ok(())
                    })?;

                let processus_manager = nodes_processus_manager.get_mut(node).unwrap();
                // get signal's color
                let color = processus_manager
                    .get_mut(&signal)
                    .expect("signal should be in processing manager");
                // update status: processed
                *color = Color::Black;

                Ok(())
            }
            // if processing: error
            Color::Grey => {
                let error = Error::NotCausalSignal {
                    node: symbol_table.get_name(*node).clone(),
                    signal: symbol_table.get_name(signal).clone(),
                    location: location.clone(),
                };
                errors.push(error);
                Err(TerminationError)
            }
            // if processed: nothing to do
            Color::Black => Ok(()),
        }
    }

    /// Add dependencies to node's inputs of a signal.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(i: int) {
    ///     out o: int = x; // depends on x which depends on input i
    ///     x: int = i;     // depends on input i
    /// }
    /// ```
    pub fn add_signal_inputs_dependencies(
        &self,
        signal: usize,
        symbol_table: &SymbolTable,
        nodes_context: &HashMap<usize, Node>,
        nodes_processus_manager: &mut HashMap<usize, HashMap<usize, Color>>,
        nodes_reduced_processus_manager: &mut HashMap<usize, HashMap<usize, Color>>,
        nodes_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        nodes_reduced_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Node { id: node, .. } = self;

        // get node's processus manager
        let processus_manager = nodes_reduced_processus_manager.get_mut(node).unwrap();
        // get signal's color
        let color = processus_manager
            .get_mut(&signal)
            .expect("signal should be in processing manager");

        match color {
            // if vertex unprocessed
            Color::White => {
                // update status: processing
                *color = Color::Grey;

                // compute signals dependencies
                self.add_signal_dependencies(
                    signal,
                    symbol_table,
                    nodes_context,
                    nodes_processus_manager,
                    nodes_reduced_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;

                // get node's graph
                let graph = nodes_graphs.get(node).unwrap().clone();

                // for every neighbors, get inputs dependencies and add it as signal dependencies
                for (_, neighbor_id, label1) in graph.edges(signal) {
                    // tells if the neighbor is an input
                    let is_input = symbol_table
                        .get_node_inputs(self.id)
                        .iter()
                        .any(|input| neighbor_id.eq(input));

                    if is_input {
                        // get node's reduced graph (borrow checker)
                        let reduced_graph = nodes_reduced_graphs.get_mut(node).unwrap();
                        // if input then add neighbor to reduced graph
                        add_edge(reduced_graph, signal, neighbor_id, label1.clone());
                        // and add its input dependencies (contract dependencies)
                        graph.edges(neighbor_id).for_each(|(_, input_id, label2)| {
                            add_edge(reduced_graph, signal, input_id, label1.add(label2))
                        });
                    } else {
                        // else compute neighbor's inputs dependencies
                        self.add_signal_inputs_dependencies(
                            neighbor_id,
                            symbol_table,
                            nodes_context,
                            nodes_processus_manager,
                            nodes_reduced_processus_manager,
                            nodes_graphs,
                            nodes_reduced_graphs,
                            errors,
                        )?;

                        // get node's reduced graph (borrow checker)
                        let reduced_graph = nodes_reduced_graphs.get_mut(node).unwrap();
                        let reduced_graph_cloned = reduced_graph.clone();

                        // add dependencies as graph's edges:
                        // s = e depends on i <=> s -> i
                        reduced_graph_cloned
                            .edges(neighbor_id)
                            .for_each(|(_, input_id, label2)| {
                                add_edge(reduced_graph, signal, input_id, label1.add(label2));
                            })
                    }
                }

                // get node's processus manager
                let processus_manager = nodes_reduced_processus_manager.get_mut(node).unwrap();
                // get signal's color
                let color = processus_manager
                    .get_mut(&signal)
                    .expect("signal should be in processing manager");
                // update status: processed
                *color = Color::Black;

                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Add signal dependencies in contract.
    ///
    /// # Example
    ///
    /// ```GR
    /// requires { j < i }  // i and j depend on each other
    /// ensures  { j < o }  // o and j depend on each other
    /// node test(i: int, j: int) {
    ///     out o: int = i;
    /// }
    /// ```
    pub fn add_contract_dependencies(
        &self,
        nodes_graphs: &mut HashMap<usize, DiGraphMap<usize, Label>>,
    ) {
        let Node {
            id: node, contract, ..
        } = self;

        // get node's graph
        let graph = nodes_graphs.get_mut(node).unwrap();

        // add edges to the graph
        // corresponding to dependencies in contract's terms
        contract.add_dependencies(graph);
    }
}
