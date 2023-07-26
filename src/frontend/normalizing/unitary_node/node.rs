use once_cell::sync::OnceCell;

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
            .map(|output| self.add_unitary_node(output))
            .collect::<Vec<_>>();

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

    fn add_unitary_node(&mut self, output: String) -> Graph<Color> {
        let Node {
            id: node,
            inputs,
            unscheduled_equations,
            unitary_nodes,
            location,
            ..
        } = self;

        // construct unitary node's subgraph from its output
        let subgraph = self.graph.get().unwrap().subgraph_from_vertex(&output);

        // get signals that compute the output
        let useful_signals = subgraph.get_vertices();

        // get useful inputs (in application order)
        let unitary_node_inputs = inputs
            .into_iter()
            .filter(|(id, _)| useful_signals.contains(id))
            .map(|input| input.clone())
            .collect::<Vec<_>>();

        // retrieve equations from useful signals
        let equations = useful_signals
            .into_iter()
            .filter_map(|signal| unscheduled_equations.get(&signal))
            .map(|equation| equation.clone())
            .collect();

        // construct unitary node
        let unitary_node = UnitaryNode {
            node_id: node.clone(),
            output_id: output.clone(),
            inputs: unitary_node_inputs,
            equations,
            memory: Memory::new(),
            location: location.clone(),
            graph: OnceCell::new(),
        };

        // insert it in node's storage
        unitary_nodes.insert(output.clone(), unitary_node);

        subgraph
    }
}

#[cfg(test)]
mod add_unitary_node {
    use once_cell::sync::OnceCell;
    use std::collections::HashMap;

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

        node.add_unitary_node(String::from("o1"));

        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("o1"),
            inputs: vec![(String::from("i1"), Type::Integer)],
            equations: vec![
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
            ],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
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

        assert!(node.eq_unscheduled(&control))
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
            equations: vec![
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
            ],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let unitary_node_2 = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("o2"),
            inputs: vec![(String::from("i2"), Type::Integer)],
            equations: vec![Equation {
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
            graph: OnceCell::new(),
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

        assert!(node.eq_unscheduled(&control));
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
