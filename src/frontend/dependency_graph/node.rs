use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::graph::color::Color;
use crate::common::graph::neighbor::Label;
use crate::error::{Error, TerminationError};
use crate::hir::contract::Contract;
use crate::hir::node::Node;

impl Node {
    /// Create an initialized graph from a node.
    ///
    /// The created graph has every node's signals as vertices.
    /// But no edges are added.
    pub fn create_initialized_graph(&self) -> DiGraphMap<String, Label> {
        // create an empty graph
        let mut graph = DiGraphMap::new();

        // get node's signals
        let Node {
            inputs,
            unscheduled_equations,
            ..
        } = self;

        // add input signals as vertices
        for (input, _) in inputs {
            graph.add_node(input);
        }

        // add other signals as vertices
        for signal in unscheduled_equations.keys() {
            graph.add_node(signal);
        }

        // return graph
        graph
    }

    /// Create an initialized processus manager from a node.
    pub fn create_initialized_processus_manager(&self) -> HashMap<&String, Color> {
        // create an empty hash
        let mut hash = HashMap::new();

        // get node's signals
        let Node {
            inputs,
            unscheduled_equations,
            ..
        } = self;

        // add input signals with white color (unprocessed)
        for (input, _) in inputs {
            hash.insert(input, Color::White);
        }

        // add other signals with white color (unprocessed)
        for signal in unscheduled_equations.keys() {
            hash.insert(signal, Color::White);
        }

        // return hash
        hash
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
        nodes_context: &HashMap<&String, Node>,
        nodes_processus_manager: &mut HashMap<String, HashMap<&String, Color>>,
        nodes_graphs: &mut HashMap<String, DiGraphMap<String, Label>>,
        nodes_reduced_graphs: &mut HashMap<String, DiGraphMap<String, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Node {
            inputs,
            unscheduled_equations,
            graph,
            ..
        } = self;

        // add local and output signals dependencies
        unscheduled_equations
            .keys()
            .map(|signal| {
                self.add_signal_dependencies(
                    signal,
                    nodes_context,
                    nodes_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )
            })
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // add input signals dependencies
        // (makes vertices colors "Black" => equal assertions in tests)
        inputs
            .iter()
            .map(|(signal, _)| {
                self.add_signal_dependencies(
                    signal,
                    nodes_context,
                    nodes_processus_manager,
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
        signal: &String,
        nodes_context: &HashMap<&String, Node>,
        nodes_processus_manager: &mut HashMap<String, HashMap<&String, Color>>,
        nodes_graphs: &mut HashMap<String, DiGraphMap<String, Label>>,
        nodes_reduced_graphs: &mut HashMap<String, DiGraphMap<String, Label>>,
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
            .get_mut(signal)
            .expect("signal should be in processing manager");

        match color {
            // if vertex unprocessed
            Color::White => {
                // update status: processing
                *color = Color::Grey;

                unscheduled_equations
                    .get(signal)
                    .map_or(Ok(()), |equation| {
                        // retrieve expression
                        let expression = &equation.expression;

                        // compute and get dependencies
                        expression.compute_dependencies(
                            nodes_context,
                            nodes_processus_manager,
                            nodes_graphs,
                            nodes_reduced_graphs,
                            errors,
                        )?;
                        let dependencies = expression.get_dependencies();

                        let graph = nodes_graphs.get_mut(node).unwrap();
                        // add dependencies as graph's edges:
                        // s = e depends on s' <=> s -> s'
                        dependencies.iter().for_each(|(id, depth)| {
                            graph.add_edge(signal, id, Label::Weight(*depth)); // TODO: warning, there might be edges
                        });

                        Ok(())
                    })?;

                let processus_manager = nodes_processus_manager.get_mut(node).unwrap();
                // get signal's color
                let color = processus_manager
                    .get_mut(signal)
                    .expect("signal should be in processing manager");
                // update status: processed
                *color = Color::Black;

                Ok(())
            }
            // if processing: error
            Color::Grey => {
                let error = Error::NotCausal {
                    node: node.clone(),
                    signal: signal.clone(),
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
        signal: &String,
        nodes_context: &HashMap<&String, Node>,
        nodes_processus_manager: &mut HashMap<String, HashMap<&String, Color>>,
        nodes_graphs: &mut HashMap<String, DiGraphMap<String, Label>>,
        nodes_reduced_graphs: &mut HashMap<String, DiGraphMap<String, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Node {
            id: node, inputs, ..
        } = self;

        // get node's processus manager
        let processus_manager = nodes_processus_manager.get_mut(node).unwrap();
        // get signal's color
        let color = processus_manager
            .get_mut(signal)
            .expect("signal should be in processing manager");

        match color {
            // if vertex unprocessed
            Color::White => {
                // update status: processing
                *color = Color::Grey;

                // compute signals dependencies
                self.add_signal_dependencies(
                    signal,
                    nodes_context,
                    nodes_processus_manager,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;

                // get node's graph
                let graph = nodes_graphs.get(node).unwrap().clone();

                // for every neighbors, get inputs dependencies and add it as signal dependencies
                for (_, neighbor_id, l1) in graph.edges(signal) {
                    // tells if the neighbor is an input
                    let is_input = inputs.iter().any(|(input, _)| input == neighbor_id);

                    if is_input {
                        // get node's reduced graph (borrow checker)
                        let reduced_graph = nodes_reduced_graphs.get_mut(node).unwrap();
                        // if input then add neighbor to reduced graph
                        reduced_graph.add_edge(signal, neighbor_id, l1.clone());
                    } else {
                        // else compute neighbor's inputs dependencies
                        self.add_signal_inputs_dependencies(
                            neighbor_id,
                            nodes_context,
                            nodes_processus_manager,
                            nodes_graphs,
                            nodes_reduced_graphs,
                            errors,
                        )?;

                        // get node's reduced graph (borrow checker)
                        let reduced_graph = nodes_reduced_graphs.get_mut(node).unwrap();
                        let reduced_graph_cloned = reduced_graph.clone();

                        // add dependencies as graph's edges:
                        // s = e depends on i <=> s -> i
                        match l1 {
                            Label::Contract => reduced_graph_cloned.edges(neighbor_id).for_each(
                                |(_, input_id, _)| {
                                    reduced_graph.add_edge(signal, input_id, Label::Contract);
                                },
                            ),
                            Label::Weight(w1) => reduced_graph_cloned.edges(neighbor_id).for_each(
                                |(_, input_id, l2)| {
                                    reduced_graph.add_edge(
                                        signal,
                                        input_id,
                                        match l2 {
                                            Label::Contract => Label::Contract,
                                            Label::Weight(w2) => Label::Weight(w1 + w2),
                                        },
                                    );
                                },
                            ),
                        }
                    }
                }

                // get node's processus manager
                let processus_manager = nodes_processus_manager.get_mut(node).unwrap();
                // get signal's color
                let color = processus_manager
                    .get_mut(signal)
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
        nodes_graphs: &mut HashMap<String, DiGraphMap<String, Label>>,
    ) {
        let Node {
            id: node,
            contract:
                Contract {
                    requires,
                    ensures,
                    invariant,
                },
            ..
        } = self;

        // get node's graph
        let graph = nodes_graphs.get_mut(node).unwrap();

        // add edges to the graph
        // corresponding to dependencies in contract's terms
        requires
            .iter()
            .for_each(|term| term.add_term_dependencies(graph));
        ensures
            .iter()
            .for_each(|term| term.add_term_dependencies(graph));
        invariant
            .iter()
            .for_each(|term| term.add_term_dependencies(graph));
    }
}

#[cfg(test)]
mod create_initialized_graph {

    use std::collections::HashMap;

    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, node::Node, once_cell::OnceCell,
        signal::Signal, stream_expression::StreamExpression,
    };

    #[test]
    fn should_initialize_graph_with_node_signals() {
        let node = Node {
            contract: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("x"),
                                scope: Scope::Local,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("x"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("x"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("i"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let graph = node.create_initialized_graph();

        assert!(graph.contains_node(&String::from("o")));
        assert!(graph.contains_node(&String::from("x")));
        assert!(graph.contains_node(&String::from("i")));
    }
}

#[cfg(test)]
mod add_all_equations_dependencies {

    use std::collections::HashMap;

    use crate::common::{graph::neighbor::Label, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, node::Node, once_cell::OnceCell,
        signal::Signal, stream_expression::StreamExpression,
    };

    #[test]
    fn should_add_all_nodes_dependencies() {
        let mut errors = vec![];

        let node = Node {
            contract: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("x"),
                                scope: Scope::Local,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("x"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("x"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("i"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut nodes_context = HashMap::new();
        let test = String::from("test");
        nodes_context.insert(&test, node);
        let node = nodes_context.get(&String::from("test")).unwrap();

        let graph = node.create_initialized_graph();
        let mut nodes_graphs = HashMap::from([(node.id.clone(), graph)]);

        let reduced_graph = node.create_initialized_graph();
        let mut nodes_reduced_graphs = HashMap::from([(node.id.clone(), reduced_graph)]);

        let processus_manager = node.create_initialized_processus_manager();
        let mut nodes_processus_manager = HashMap::from([(node.id.clone(), processus_manager)]);

        node.add_all_equations_dependencies(
            &nodes_context,
            &mut nodes_processus_manager,
            &mut nodes_graphs,
            &mut nodes_reduced_graphs,
            &mut errors,
        )
        .unwrap();

        let graph = nodes_graphs.get(&node.id).unwrap();

        assert!(graph.contains_node(&String::from("o")));
        assert!(graph.contains_node(&String::from("x")));
        assert!(graph.contains_node(&String::from("i")));
        assert_eq!(
            graph.edge_weight(&String::from("x"), &String::from("i")),
            Some(Label::Weight(0)).as_ref()
        );
        assert_eq!(
            graph.edge_weight(&String::from("o"), &String::from("x")),
            Some(Label::Weight(0)).as_ref()
        );
    }
}

#[cfg(test)]
mod add_signal_dependencies {

    use std::collections::HashMap;

    use crate::common::{graph::neighbor::Label, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, node::Node, once_cell::OnceCell,
        signal::Signal, stream_expression::StreamExpression,
    };

    #[test]
    fn should_add_dependencies_of_a_signal_of_the_node_to_graph() {
        let mut errors = vec![];

        let node = Node {
            contract: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("x"),
                                scope: Scope::Local,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("x"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("x"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("i"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut nodes_context = HashMap::new();
        let test = String::from("test");
        nodes_context.insert(&test, node);
        let node = nodes_context.get(&String::from("test")).unwrap();

        let graph = node.create_initialized_graph();
        let mut nodes_graphs = HashMap::from([(node.id.clone(), graph)]);

        let reduced_graph = node.create_initialized_graph();
        let mut nodes_reduced_graphs = HashMap::from([(node.id.clone(), reduced_graph)]);

        let processus_manager = node.create_initialized_processus_manager();
        let mut nodes_processus_manager = HashMap::from([(node.id.clone(), processus_manager)]);

        let x = String::from("x");
        node.add_signal_dependencies(
            &x,
            &nodes_context,
            &mut nodes_processus_manager,
            &mut nodes_graphs,
            &mut nodes_reduced_graphs,
            &mut errors,
        )
        .unwrap();

        let graph = nodes_graphs.get(&node.id).unwrap();

        assert!(graph.contains_node(&String::from("o")));
        assert!(graph.contains_node(&String::from("x")));
        assert!(graph.contains_node(&String::from("i")));
        assert_eq!(
            graph.edge_weight(&String::from("x"), &String::from("i")),
            Some(Label::Weight(0)).as_ref()
        );
    }
}

#[cfg(test)]
mod add_signal_inputs_dependencies {

    use std::collections::HashMap;

    use crate::common::{graph::neighbor::Label, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, node::Node, once_cell::OnceCell,
        signal::Signal, stream_expression::StreamExpression,
    };

    #[test]
    fn should_add_inputs_dependencies_of_a_signal_of_the_node_to_reduced_graph() {
        let mut errors = vec![];

        let node = Node {
            contract: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("x"),
                                scope: Scope::Local,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("x"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("x"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("i"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut nodes_context = HashMap::new();
        let test = String::from("test");
        nodes_context.insert(&test, node);
        let node = nodes_context.get(&String::from("test")).unwrap();

        let graph = node.create_initialized_graph();
        let mut nodes_graphs = HashMap::from([(node.id.clone(), graph)]);

        let reduced_graph = node.create_initialized_graph();
        let mut nodes_reduced_graphs = HashMap::from([(node.id.clone(), reduced_graph)]);

        let processus_manager = node.create_initialized_processus_manager();
        let mut nodes_processus_manager = HashMap::from([(node.id.clone(), processus_manager)]);

        let o = String::from("o");
        node.add_signal_inputs_dependencies(
            &o,
            &nodes_context,
            &mut nodes_processus_manager,
            &mut nodes_graphs,
            &mut nodes_reduced_graphs,
            &mut errors,
        )
        .unwrap();

        let reduced_graph = nodes_reduced_graphs.get(&node.id).unwrap();

        assert!(reduced_graph.contains_node(&String::from("o")));
        assert!(reduced_graph.contains_node(&String::from("x")));
        assert!(reduced_graph.contains_node(&String::from("i")));
        assert_eq!(
            reduced_graph.edge_weight(&String::from("x"), &String::from("i")),
            Some(Label::Weight(0)).as_ref()
        );
        assert_eq!(
            reduced_graph.edge_weight(&String::from("o"), &String::from("i")),
            Some(Label::Weight(0)).as_ref()
        );
    }
}
