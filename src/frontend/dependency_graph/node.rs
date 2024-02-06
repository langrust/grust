use std::collections::HashMap;

use crate::common::graph::neighbor::Label;
use crate::common::graph::{color::Color, neighbor::Neighbor, Graph};
use crate::error::{Error, TerminationError};
use crate::hir::node::Node;

impl Node {
    /// Create an initialized graph from a node.
    ///
    /// The created graph has every node's signals as vertices.
    /// But no edges are added.
    pub fn create_initialized_graph(&self) -> Graph<Color> {
        // create an empty graph
        let mut graph = Graph::new();

        // get node's signals
        let Node {
            inputs,
            unscheduled_equations,
            ..
        } = self;

        // add input signals as vertices
        for (input, _) in inputs {
            graph.add_vertex(input.clone(), Color::White);
        }

        // add other signals as vertices
        for signal in unscheduled_equations.keys() {
            graph.add_vertex(signal.clone(), Color::White);
        }

        // return graph
        graph
    }

    /// Complete dependency graph of the node.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(i: int) { // i depends on nothing
    ///     out o: int = x; // depends on x
    ///     x: int = i;     // depends on i
    /// }
    /// ```
    pub fn add_all_dependencies(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
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
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Node {
            id: node,
            unscheduled_equations,
            location,
            ..
        } = self;

        // get node's graph
        let graph = nodes_graphs.get_mut(node).unwrap();
        // get signal's vertex
        let vertex = graph.get_vertex_mut(signal);

        match vertex.get_value() {
            // if vertex unprocessed
            Color::White => {
                // update status: processing
                vertex.set_value(Color::Grey);

                unscheduled_equations
                    .get(signal)
                    .map_or(Ok(()), |equation| {
                        // retrieve expression
                        let expression = &equation.expression;

                        // compute and get dependencies
                        expression.compute_dependencies(
                            nodes_context,
                            nodes_graphs,
                            nodes_reduced_graphs,
                            errors,
                        )?;
                        let dependencies = expression.get_dependencies().clone();

                        // get node's graph (borrow checker)
                        let graph = nodes_graphs.get_mut(node).unwrap();

                        // add dependencies as graph's edges:
                        // s = e depends on s' <=> s -> s'
                        dependencies
                            .iter()
                            .for_each(|(id, depth)| graph.add_weighted_edge(signal, id.clone(), *depth));

                        Ok(())
                    })?;

                // get node's graph (borrow checker)
                let graph = nodes_graphs.get_mut(node).unwrap();
                // get signal's vertes (borrow checker)
                let vertex = graph.get_vertex_mut(signal);
                // update status: processed
                vertex.set_value(Color::Black);

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
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Node {
            id: node, inputs, ..
        } = self;

        // get node's reduced graph
        let reduced_graph = nodes_reduced_graphs.get_mut(node).unwrap();
        // get signal's vertex
        let reduced_vertex = reduced_graph.get_vertex_mut(signal);

        match reduced_vertex.get_value() {
            // if vertex unprocessed
            Color::White => {
                // update status: processing
                reduced_vertex.set_value(Color::Grey);

                // compute signals dependencies
                self.add_signal_dependencies(
                    signal,
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;

                // get node's graph
                let graph = nodes_graphs.get_mut(node).unwrap();
                // get signal's vertex
                let vertex = graph.get_vertex_mut(signal);

                // for every neighbors, get inputs dependencies
                for Neighbor { id, label: l1 } in vertex.get_neighbors() {
                    // tells if the neighbor is an input
                    let is_input = inputs.iter().any(|(input, _)| *input == id);

                    if is_input {
                        // get node's reduced graph (borrow checker)
                        let reduced_graph = nodes_reduced_graphs.get_mut(node).unwrap();
                        // if input then add neighbor to reduced graph
                        reduced_graph.add_edge(signal, id, l1);
                    } else {
                        // else compute neighbor's inputs dependencies
                        self.add_signal_inputs_dependencies(
                            &id,
                            nodes_context,
                            nodes_graphs,
                            nodes_reduced_graphs,
                            errors,
                        )?;

                        // get node's reduced graph (borrow checker)
                        let reduced_graph = nodes_reduced_graphs.get_mut(node).unwrap();
                        // get neighbor's vertex
                        let reduced_vertex = reduced_graph.get_vertex(&id);

                        // add dependencies as graph's edges:
                        // s = e depends on i <=> s -> i
                        match l1 {
                            Label::Contract => reduced_vertex.get_neighbors().into_iter().for_each(
                                |Neighbor { id, label: _ }| {
                                    reduced_graph.add_edge(signal, id, Label::Contract)
                                },
                            ),
                            Label::Weight(w1) => reduced_vertex.get_neighbors().into_iter().for_each(
                                |Neighbor { id, label: l2 }| {
                                    match l2 {
                                       Label::Contract => reduced_graph.add_edge(signal, id, Label::Contract),
                                       Label::Weight(w2) => reduced_graph.add_edge(signal, id, Label::Weight(w1 + w2)),
                                    }
                                },
                            ),
                        }
                    }
                }

                // get node's reduced graph (borrow checker)
                let reduced_graph = nodes_reduced_graphs.get_mut(node).unwrap();
                // get signal's vertex (borrow checker)
                let reduced_vertex = reduced_graph.get_vertex_mut(signal);
                reduced_vertex.set_value(Color::Black);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod create_initialized_graph {

    use std::collections::HashMap;

    use crate::common::{
        graph::{color::Color, Graph},
        location::Location,
        r#type::Type,
        scope::Scope,
    };
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, node::Node, once_cell::OnceCell,
        signal::Signal, stream_expression::StreamExpression,
    };

    #[test]
    fn should_initialize_graph_with_node_signals() {
        let node = Node {
            contracts: Default::default(),
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

        let mut control = Graph::new();
        control.add_vertex(String::from("o"), Color::White);
        control.add_vertex(String::from("x"), Color::White);
        control.add_vertex(String::from("i"), Color::White);

        assert_eq!(graph, control);
    }
}

#[cfg(test)]
mod add_all_dependencies {

    use std::collections::HashMap;

    use crate::common::{
        graph::{color::Color, Graph},
        location::Location,
        r#type::Type,
        scope::Scope,
    };
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, node::Node, once_cell::OnceCell,
        signal::Signal, stream_expression::StreamExpression,
    };

    #[test]
    fn should_add_all_nodes_dependencies() {
        let mut errors = vec![];

        let node = Node {
            contracts: Default::default(),
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
        nodes_context.insert(String::from("test"), node);
        let node = nodes_context.get(&String::from("test")).unwrap();

        let graph = node.create_initialized_graph();
        let mut nodes_graphs = HashMap::from([(node.id.clone(), graph)]);

        let reduced_graph = node.create_initialized_graph();
        let mut nodes_reduced_graphs = HashMap::from([(node.id.clone(), reduced_graph)]);

        node.add_all_dependencies(
            &nodes_context,
            &mut nodes_graphs,
            &mut nodes_reduced_graphs,
            &mut errors,
        )
        .unwrap();

        let graph = nodes_graphs.get(&node.id).unwrap();

        let mut control = Graph::new();
        control.add_vertex(String::from("o"), Color::Black);
        control.add_vertex(String::from("x"), Color::Black);
        control.add_vertex(String::from("i"), Color::Black);
        control.add_weighted_edge(&String::from("x"), String::from("i"), 0);
        control.add_weighted_edge(&String::from("o"), String::from("x"), 0);

        assert_eq!(*graph, control);
    }
}

#[cfg(test)]
mod add_signal_dependencies {

    use std::collections::HashMap;

    use crate::common::{
        graph::{color::Color, Graph},
        location::Location,
        r#type::Type,
        scope::Scope,
    };
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, node::Node, once_cell::OnceCell,
        signal::Signal, stream_expression::StreamExpression,
    };

    #[test]
    fn should_add_dependencies_of_a_signal_of_the_node_to_graph() {
        let mut errors = vec![];

        let node = Node {
            contracts: Default::default(),
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
        nodes_context.insert(String::from("test"), node);
        let node = nodes_context.get(&String::from("test")).unwrap();

        let graph = node.create_initialized_graph();
        let mut nodes_graphs = HashMap::from([(node.id.clone(), graph)]);

        let reduced_graph = node.create_initialized_graph();
        let mut nodes_reduced_graphs = HashMap::from([(node.id.clone(), reduced_graph)]);

        node.add_signal_dependencies(
            &String::from("x"),
            &nodes_context,
            &mut nodes_graphs,
            &mut nodes_reduced_graphs,
            &mut errors,
        )
        .unwrap();

        let graph = nodes_graphs.get(&node.id).unwrap();

        let mut control = Graph::new();
        control.add_vertex(String::from("o"), Color::White);
        control.add_vertex(String::from("x"), Color::Black);
        control.add_vertex(String::from("i"), Color::White);
        control.add_weighted_edge(&String::from("x"), String::from("i"), 0);

        assert_eq!(*graph, control);
    }
}

#[cfg(test)]
mod add_signal_inputs_dependencies {

    use std::collections::HashMap;

    use crate::common::{
        graph::{color::Color, Graph},
        location::Location,
        r#type::Type,
        scope::Scope,
    };
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, node::Node, once_cell::OnceCell,
        signal::Signal, stream_expression::StreamExpression,
    };

    #[test]
    fn should_add_inputs_dependencies_of_a_signal_of_the_node_to_reduced_graph() {
        let mut errors = vec![];

        let node = Node {
            contracts: Default::default(),
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
        nodes_context.insert(String::from("test"), node);
        let node = nodes_context.get(&String::from("test")).unwrap();

        let graph = node.create_initialized_graph();
        let mut nodes_graphs = HashMap::from([(node.id.clone(), graph)]);

        let reduced_graph = node.create_initialized_graph();
        let mut nodes_reduced_graphs = HashMap::from([(node.id.clone(), reduced_graph)]);

        node.add_signal_inputs_dependencies(
            &String::from("o"),
            &nodes_context,
            &mut nodes_graphs,
            &mut nodes_reduced_graphs,
            &mut errors,
        )
        .unwrap();

        let reduced_graph = nodes_reduced_graphs.get(&node.id).unwrap();

        let mut control = Graph::new();
        control.add_vertex(String::from("o"), Color::Black);
        control.add_vertex(String::from("x"), Color::Black);
        control.add_vertex(String::from("i"), Color::White);
        control.add_weighted_edge(&String::from("x"), String::from("i"), 0);
        control.add_weighted_edge(&String::from("o"), String::from("i"), 0);

        assert_eq!(*reduced_graph, control);
    }
}
