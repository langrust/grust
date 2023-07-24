use crate::common::{
    graph::{color::Color, Graph},
    scope::Scope,
};
use crate::error::Error;
use crate::hir::{memory::Memory, node::Node, unitary_node::UnitaryNode};

impl Node {
    /// Generate unitary nodes from mother node.
    ///
    /// Generate and add unitary nodes to mother node.
    /// Unitary nodes are nodes with one output and contains
    /// all signals from which the output computation depends.
    ///
    /// Unitary nodes computations induces schedulings of the node.
    /// This detects causality errors.
    ///
    /// It also detects unused signal definitions or inputs.
    pub fn generate_unitary_nodes(&mut self, errors: &mut Vec<Error>) -> Result<(), ()> {
        // get outputs identifiers
        let outputs = self
            .unscheduled_equations
            .values()
            .filter(|equation| equation.scope.eq(&Scope::Output))
            .map(|equation| equation.id.clone())
            .collect::<Vec<_>>();

        // construct unitary node for each output
        let subgraphs = outputs
            .into_iter()
            .map(|output| self.add_unitary_node(output, errors))
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Result<Vec<_>, ()>>()?;

        // check that every signals are used
        let unused_signals = self.graph.get().unwrap().forgotten_vertices(subgraphs);
        unused_signals
            .into_iter()
            .map(|signal| {
                let error = Error::UnusedSignal {
                    node: self.id.clone(),
                    signal,
                    location: self.location.clone(),
                };
                errors.push(error);
                Err(())
            })
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Result<_, _>>()
    }

    fn add_unitary_node(
        &mut self,
        output: String,
        errors: &mut Vec<Error>,
    ) -> Result<Graph<Color>, ()> {
        let Node {
            id: node,
            inputs,
            unscheduled_equations,
            unitary_nodes,
            location,
            ..
        } = self;

        // construct unitary node's subgraph from its output, containing
        // only 0-depth dependencies
        let mut subgraph = self
            .graph
            .get()
            .unwrap()
            .subgraph_from_vertex(&output)
            .subgraph_on_edges(|weight| weight == 0);

        // schedule the unitary node
        let schedule = subgraph.topological_sorting(errors).map_err(|signal| {
            let error = Error::NotCausal {
                node: node.clone(),
                signal: signal,
                location: location.clone(),
            };
            errors.push(error)
        })?;

        // get usefull inputs (in application order)
        let unitary_node_inputs = inputs
            .into_iter()
            .filter(|(id, _)| schedule.contains(id))
            .map(|input| input.clone())
            .collect::<Vec<_>>();

        // retrieve scheduled equations from schedule
        // and mother node's equations
        let scheduled_equations = schedule
            .into_iter()
            .filter_map(|signal| unscheduled_equations.get(&signal))
            .map(|equation| equation.clone())
            .collect();

        // construct unitary node
        let unitary_node = UnitaryNode {
            node_id: node.clone(),
            output_id: output.clone(),
            inputs: unitary_node_inputs,
            scheduled_equations,
            memory: Memory::new(),
            location: location.clone(),
        };

        // insert it in node's storage
        unitary_nodes.insert(output.clone(), unitary_node);

        Ok(subgraph)
    }
}

#[cfg(test)]
mod add_unitary_node {
    use once_cell::sync::OnceCell;
    use std::collections::HashMap;

    use crate::common::constant::Constant;
    use crate::hir::{
        equation::Equation, memory::Memory, node::Node, stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    };
    use crate::{
        common::{
            graph::{color::Color, Graph},
            location::Location,
            r#type::Type,
            scope::Scope,
        },
        hir::dependencies::Dependencies,
    };

    #[test]
    fn should_add_unitary_node_computing_output() {
        let mut errors = vec![];

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i2"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                            id: String::from("i1"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("i1"), 0);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o2"), String::from("i2"), 0);

        node.graph.set(graph).unwrap();

        node.add_unitary_node(String::from("o1"), &mut errors)
            .unwrap();

        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("o1"),
            inputs: vec![(String::from("i1"), Type::Integer)],
            scheduled_equations: vec![
                Equation {
                    scope: Scope::Local,
                    id: String::from("x"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        id: String::from("i1"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                    },
                    location: Location::default(),
                },
                Equation {
                    scope: Scope::Output,
                    id: String::from("o1"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                    location: Location::default(),
                },
            ],
            memory: Memory::new(),
            location: Location::default(),
        };
        let control = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i2"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                            id: String::from("i1"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::from([(String::from("o1"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("i1"), 0);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o2"), String::from("i2"), 0);
        control.graph.set(graph.clone()).unwrap();

        assert_eq!(node, control)
    }

    #[test]
    fn should_be_scheduled() {
        let mut errors = vec![];

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i2"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                            id: String::from("i1"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("i1"), 0);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o2"), String::from("i2"), 0);

        node.graph.set(graph.clone()).unwrap();

        node.add_unitary_node(String::from("o1"), &mut errors)
            .unwrap();

        let unitary_node = node.unitary_nodes.get(&String::from("o1")).unwrap();
        let schedule = unitary_node
            .scheduled_equations
            .iter()
            .map(|equation| &equation.id)
            .collect::<Vec<_>>();

        let test = graph
            .get_edges()
            .iter()
            .filter_map(|(v1, v2, _)| {
                schedule
                    .iter()
                    .position(|id| id.eq(&v1))
                    .map(|i1| schedule.iter().position(|id| id.eq(&v2)).map(|i2| (i1, i2)))
            })
            .filter_map(|o| o)
            .all(|(i1, i2)| i2 <= i1);

        assert!(test)
    }

    #[test]
    fn should_inform_of_causality_error() {
        let mut errors = vec![];

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i2"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                            id: String::from("o1"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("o1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("o1"), 0);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o2"), String::from("i2"), 0);

        node.graph.set(graph).unwrap();

        node.add_unitary_node(String::from("o1"), &mut errors)
            .unwrap_err()
    }

    #[test]
    fn should_add_unitary_node_in_case_of_shifted_causality_loop() {
        let mut errors = vec![];

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
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
                        expression: StreamExpression::FollowedBy {
                            constant: Constant::Integer(0),
                            expression: Box::new(StreamExpression::SignalCall {
                                id: String::from("o1"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("o1"), 0)]),
                            }),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("o1"), 1)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("o1"), 1);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o2"), String::from("i"), 0);

        node.graph.set(graph).unwrap();

        node.add_unitary_node(String::from("o1"), &mut errors)
            .unwrap();

        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("o1"),
            inputs: vec![],
            scheduled_equations: vec![
                Equation {
                    scope: Scope::Local,
                    id: String::from("x"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::SignalCall {
                            id: String::from("o1"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("o1"), 0)]),
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("o1"), 1)]),
                    },
                    location: Location::default(),
                },
                Equation {
                    scope: Scope::Output,
                    id: String::from("o1"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                    location: Location::default(),
                },
            ],
            memory: Memory::new(),
            location: Location::default(),
        };
        let control = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
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
                        expression: StreamExpression::FollowedBy {
                            constant: Constant::Integer(0),
                            expression: Box::new(StreamExpression::SignalCall {
                                id: String::from("o1"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("o1"), 0)]),
                            }),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("o1"), 1)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::from([(String::from("o1"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("o1"), 1);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o2"), String::from("i"), 0);
        control.graph.set(graph.clone()).unwrap();

        assert_eq!(node, control)
    }
}

#[cfg(test)]
mod generate_unitary_nodes {
    use once_cell::sync::OnceCell;

    use crate::hir::{
        equation::Equation, memory::Memory, node::Node, stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    };
    use crate::{
        common::{
            graph::{color::Color, Graph},
            location::Location,
            r#type::Type,
            scope::Scope,
        },
        hir::dependencies::Dependencies,
    };
    use std::collections::HashMap;

    #[test]
    fn should_generate_unitary_nodes_as_expected() {
        let mut errors = vec![];

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i2"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                            id: String::from("i1"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("i1"), 0);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o2"), String::from("i2"), 0);

        node.graph.set(graph).unwrap();

        node.generate_unitary_nodes(&mut errors).unwrap();

        let unitary_node_1 = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("o1"),
            inputs: vec![(String::from("i1"), Type::Integer)],
            scheduled_equations: vec![
                Equation {
                    scope: Scope::Local,
                    id: String::from("x"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        id: String::from("i1"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                    },
                    location: Location::default(),
                },
                Equation {
                    scope: Scope::Output,
                    id: String::from("o1"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                    location: Location::default(),
                },
            ],
            memory: Memory::new(),
            location: Location::default(),
        };
        let unitary_node_2 = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("o2"),
            inputs: vec![(String::from("i2"), Type::Integer)],
            scheduled_equations: vec![Equation {
                scope: Scope::Output,
                id: String::from("o2"),
                signal_type: Type::Integer,
                expression: StreamExpression::SignalCall {
                    id: String::from("i2"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
                },
                location: Location::default(),
            }],
            memory: Memory::new(),
            location: Location::default(),
        };
        let control = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i2"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                            id: String::from("i1"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::from([
                (String::from("o2"), unitary_node_2),
                (String::from("o1"), unitary_node_1),
            ]),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("i1"), 0);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o2"), String::from("i2"), 0);

        control.graph.set(graph.clone()).unwrap();

        assert_eq!(node, control)
    }

    #[test]
    fn should_generate_unitary_nodes_for_every_output() {
        let mut errors = vec![];

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i2"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                            id: String::from("i1"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("i1"), 0);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o2"), String::from("i2"), 0);

        node.graph.set(graph).unwrap();

        node.generate_unitary_nodes(&mut errors).unwrap();

        let mut output_equations = node
            .unscheduled_equations
            .iter()
            .filter(|(_, equation)| equation.scope.eq(&Scope::Output));

        assert!(output_equations.all(|(signal, _)| node.unitary_nodes.contains_key(signal)))
    }

    #[test]
    fn should_raise_error_when_not_causal() {
        let mut errors = vec![];

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i2"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                            id: String::from("o1"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("o1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("o1"), 0);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o2"), String::from("i2"), 0);

        node.graph.set(graph).unwrap();

        node.generate_unitary_nodes(&mut errors).unwrap_err()
    }

    #[test]
    fn should_raise_error_for_unused_signals() {
        let mut errors = vec![];

        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i1"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
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
                            id: String::from("i1"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("i1"), Color::Black);
        graph.add_vertex(String::from("i2"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("i1"), 0);
        graph.add_edge(&String::from("o1"), String::from("i1"), 0);

        node.graph.set(graph).unwrap();

        node.generate_unitary_nodes(&mut errors).unwrap_err()
    }
}
