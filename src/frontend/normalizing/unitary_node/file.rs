use std::collections::HashMap;

use crate::error::{Error, TerminationError};
use crate::hir::file::File;

impl File {
    /// Generate unitary nodes.
    ///
    /// It also changes node application expressions into unitary node application
    /// and removes unused inputs from those unitary node application.
    ///
    /// # Example
    ///
    /// Let be a node `my_node` and a node `other_node` as follows:
    ///
    /// ```GR
    /// node my_node(x: int, y: int) {
    ///     out o1: int = x+y;
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int, g: int) {
    ///     out x: int = my_node(y, v).o1;
    ///     y: int = my_node(g-1, v).o2;
    /// }
    /// ```
    ///
    /// The generated unitary nodes are the following:
    ///
    /// ```GR
    /// node my_node(x: int, y: int).o1 {
    ///     out o1: int = x+y;
    /// }
    /// node my_node(y: int).o2 {
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int) {           // g is then unused and will raise an error
    ///     out x: int = my_node(y, v).o1;
    ///     y: int = my_node(v).o2;
    /// }
    /// ```
    pub fn generate_unitary_nodes(
        &mut self,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        // unitary nodes computations, it induces unused signals tracking
        self.nodes
            .iter_mut()
            .map(|node| node.generate_unitary_nodes(errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // get, for each unitary node, initial node's inputs
        // that are used by the unitary node
        let mut unitary_nodes_used_inputs = HashMap::new();
        self.nodes.iter().for_each(|node| {
            node.unitary_nodes
                .iter()
                .for_each(|(output, unitary_node)| {
                    assert!(unitary_nodes_used_inputs
                        .insert(
                            (node.id.clone(), output.clone()),
                            node.inputs
                                .iter()
                                .map(|input| {
                                    (input.0.clone(), unitary_node.inputs.contains(input))
                                })
                                .collect::<Vec<_>>(),
                        )
                        .is_none());
                })
        });

        // change node application to unitary node application
        self.nodes.iter_mut().for_each(|node| {
            node.unitary_nodes.values_mut().for_each(|unitary_node| {
                unitary_node.equations.iter_mut().for_each(|equation| {
                    equation
                        .expression
                        .change_node_application_into_unitary_node_application(
                            &unitary_nodes_used_inputs,
                        )
                })
            })
        });

        Ok(())
    }
}

#[cfg(test)]
mod generate_unitary_nodes {
    use once_cell::sync::OnceCell;

    use crate::ast::{expression::Expression, function::Function, statement::Statement};
    use crate::common::graph::{color::Color, Graph};
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, file::File, memory::Memory, node::Node,
        signal::Signal, stream_expression::StreamExpression, unitary_node::UnitaryNode,
    };
    use std::collections::HashMap;

    #[test]
    fn should_generate_unitary_nodes_from_nodes() {
        let mut errors = vec![];

        // my_node(x: int, y: int) { out o1: int = x+y; out o2: int = 2*y; }
        let node = Node {
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("+"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("x"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                                },
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("y"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![
                                (String::from("x"), 0),
                                (String::from("y"), 0),
                            ]),
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
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("*"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::Constant {
                                    constant: Constant::Integer(2),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![]),
                                },
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("y"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
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
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("y"), Color::Black);
        graph.add_vertex(String::from("o1"), Color::Black);
        graph.add_vertex(String::from("o2"), Color::Black);
        graph.add_edge(&String::from("o1"), String::from("x"), 0);
        graph.add_edge(&String::from("o1"), String::from("y"), 0);
        graph.add_edge(&String::from("o2"), String::from("y"), 0);
        node.graph.set(graph.clone()).unwrap();

        let mut file = File {
            typedefs: vec![],
            functions: vec![],
            nodes: vec![node],
            component: None,
            location: Location::default(),
        };
        file.generate_unitary_nodes(&mut errors).unwrap();

        let node_control = Node {
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("+"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("x"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                                },
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("y"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![
                                (String::from("x"), 0),
                                (String::from("y"), 0),
                            ]),
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
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("*"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::Constant {
                                    constant: Constant::Integer(2),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![]),
                                },
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("y"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::from([
                (
                    String::from("o1"),
                    UnitaryNode {
                        node_id: String::from("my_node"),
                        output_id: String::from("o1"),
                        inputs: vec![
                            (String::from("x"), Type::Integer),
                            (String::from("y"), Type::Integer),
                        ],
                        equations: vec![Equation {
                            scope: Scope::Output,
                            id: String::from("o1"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: String::from("+"),
                                    typing: Some(Type::Abstract(
                                        vec![Type::Integer, Type::Integer],
                                        Box::new(Type::Integer),
                                    )),
                                    location: Location::default(),
                                },
                                inputs: vec![
                                    StreamExpression::SignalCall {
                                        signal: Signal {
                                            id: String::from("x"),
                                            scope: Scope::Input,
                                        },
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::from(vec![(
                                            String::from("x"),
                                            0,
                                        )]),
                                    },
                                    StreamExpression::SignalCall {
                                        signal: Signal {
                                            id: String::from("y"),
                                            scope: Scope::Input,
                                        },
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::from(vec![(
                                            String::from("y"),
                                            0,
                                        )]),
                                    },
                                ],
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![
                                    (String::from("x"), 0),
                                    (String::from("y"), 0),
                                ]),
                            },
                            location: Location::default(),
                        }],
                        memory: Memory::new(),
                        location: Location::default(),
                        graph: OnceCell::new(),
                    },
                ),
                (
                    String::from("o2"),
                    UnitaryNode {
                        node_id: String::from("my_node"),
                        output_id: String::from("o2"),
                        inputs: vec![(String::from("y"), Type::Integer)],
                        equations: vec![Equation {
                            scope: Scope::Output,
                            id: String::from("o2"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: String::from("*"),
                                    typing: Some(Type::Abstract(
                                        vec![Type::Integer, Type::Integer],
                                        Box::new(Type::Integer),
                                    )),
                                    location: Location::default(),
                                },
                                inputs: vec![
                                    StreamExpression::Constant {
                                        constant: Constant::Integer(2),
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::from(vec![]),
                                    },
                                    StreamExpression::SignalCall {
                                        signal: Signal {
                                            id: String::from("y"),
                                            scope: Scope::Input,
                                        },
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::from(vec![(
                                            String::from("y"),
                                            0,
                                        )]),
                                    },
                                ],
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                            },
                            location: Location::default(),
                        }],
                        memory: Memory::new(),
                        location: Location::default(),
                        graph: OnceCell::new(),
                    },
                ),
            ]),
            location: Location::default(),
            graph: OnceCell::from(graph),
        };

        let control = File {
            typedefs: vec![],
            functions: vec![],
            nodes: vec![node_control],
            component: None,
            location: Location::default(),
        };

        println!("{:?}", file.nodes[0]);
        println!("{:?}", control.nodes[0]);
        assert!(file.nodes[0].eq_unscheduled(&control.nodes[0]));
    }

    #[test]
    fn should_change_node_application_to_unitary_node_application() {
        let mut errors = vec![];

        // my_node(x: int, y: int) { out o: int = x*y; }
        let my_node = Node {
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                Equation {
                    scope: Scope::Output,
                    id: String::from("o"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("*"),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer, Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![
                            StreamExpression::SignalCall {
                                signal: Signal {
                                    id: String::from("x"),
                                    scope: Scope::Input,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                            },
                            StreamExpression::SignalCall {
                                signal: Signal {
                                    id: String::from("y"),
                                    scope: Scope::Input,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                            },
                        ],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![
                            (String::from("x"), 0),
                            (String::from("y"), 0),
                        ]),
                    },
                    location: Location::default(),
                },
            )]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut my_node_graph = Graph::new();
        my_node_graph.add_vertex(String::from("x"), Color::Black);
        my_node_graph.add_vertex(String::from("y"), Color::Black);
        my_node_graph.add_vertex(String::from("o"), Color::Black);
        my_node_graph.add_edge(&String::from("o"), String::from("x"), 0);
        my_node_graph.add_edge(&String::from("o"), String::from("y"), 0);
        my_node.graph.set(my_node_graph.clone()).unwrap();

        // other_node(x: int, y: int) { out o1: int = x+y; out o2: int = 2*y; }
        let other_node = Node {
            id: String::from("other_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("+"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("x"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                                },
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("y"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![
                                (String::from("x"), 0),
                                (String::from("y"), 0),
                            ]),
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
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("*"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::Constant {
                                    constant: Constant::Integer(2),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![]),
                                },
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("y"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut other_node_graph = Graph::new();
        other_node_graph.add_vertex(String::from("x"), Color::Black);
        other_node_graph.add_vertex(String::from("y"), Color::Black);
        other_node_graph.add_vertex(String::from("o1"), Color::Black);
        other_node_graph.add_vertex(String::from("o2"), Color::Black);
        other_node_graph.add_edge(&String::from("o1"), String::from("x"), 0);
        other_node_graph.add_edge(&String::from("o1"), String::from("y"), 0);
        other_node_graph.add_edge(&String::from("o2"), String::from("y"), 0);
        other_node.graph.set(other_node_graph.clone()).unwrap();

        // out x: int = 1 + my_node(s, v*2).o
        let equation_1 = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: String::from("+"),
                    typing: Some(Type::Abstract(
                        vec![Type::Integer, Type::Integer],
                        Box::new(Type::Integer),
                    )),
                    location: Location::default(),
                },
                inputs: vec![
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![]),
                    },
                    StreamExpression::NodeApplication {
                        node: String::from("my_node"),
                        inputs: vec![
                            StreamExpression::SignalCall {
                                signal: Signal {
                                    id: String::from("s"),
                                    scope: Scope::Input,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                            },
                            StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: String::from("*2"),
                                    typing: Some(Type::Abstract(
                                        vec![Type::Integer],
                                        Box::new(Type::Integer),
                                    )),
                                    location: Location::default(),
                                },
                                inputs: vec![StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("v"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                                }],
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                            },
                        ],
                        signal: String::from("o"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![
                            (String::from("s"), 0),
                            (String::from("v"), 0),
                        ]),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("s"), 0),
                    (String::from("v"), 0),
                ]),
            },
            location: Location::default(),
        };
        // out y: int = other_node(g-1, v).o1
        let equation_2 = Equation {
            scope: Scope::Output,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::NodeApplication {
                node: String::from("other_node"),
                inputs: vec![
                    StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("-1"),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("g"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("g"), 0)]),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("g"), 0)]),
                    },
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("v"),
                            scope: Scope::Input,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                    },
                ],
                signal: String::from("o1"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("g"), 0),
                    (String::from("v"), 0),
                ]),
            },
            location: Location::default(),
        };
        // out z: int = other_node(g-1, v).o2
        let equation_3 = Equation {
            scope: Scope::Output,
            id: String::from("z"),
            signal_type: Type::Integer,
            expression: StreamExpression::NodeApplication {
                node: String::from("other_node"),
                inputs: vec![
                    StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("-1"),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("g"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::new(),
                    },
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("v"),
                            scope: Scope::Input,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                    },
                ],
                signal: String::from("o2"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
            },
            location: Location::default(),
        };
        let node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1.clone()),
                (String::from("y"), equation_2.clone()),
                (String::from("z"), equation_3.clone()),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut node_graph = Graph::new();
        node_graph.add_vertex(String::from("s"), Color::Black);
        node_graph.add_vertex(String::from("v"), Color::Black);
        node_graph.add_vertex(String::from("g"), Color::Black);
        node_graph.add_vertex(String::from("x"), Color::Black);
        node_graph.add_vertex(String::from("y"), Color::Black);
        node_graph.add_vertex(String::from("z"), Color::Black);
        node_graph.add_edge(&String::from("x"), String::from("s"), 0);
        node_graph.add_edge(&String::from("x"), String::from("v"), 0);
        node_graph.add_edge(&String::from("y"), String::from("g"), 0);
        node_graph.add_edge(&String::from("y"), String::from("v"), 0);
        node_graph.add_edge(&String::from("z"), String::from("v"), 0);
        node.graph.set(node_graph.clone()).unwrap();

        let function = Function {
            id: String::from("my_function"),
            inputs: vec![(String::from("i"), Type::Integer)],
            statements: vec![Statement {
                id: String::from("x"),
                element_type: Type::Integer,
                expression: Expression::Call {
                    id: String::from("i"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                location: Location::default(),
            }],
            returned: (
                Type::Integer,
                Expression::Call {
                    id: String::from("x"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };
        let mut file = File {
            typedefs: vec![],
            functions: vec![function],
            nodes: vec![my_node, other_node, node],
            component: None,
            location: Location::default(),
        };
        file.generate_unitary_nodes(&mut errors).unwrap();

        // my_node(x: int, y: int) { out o: int = x*y; }
        let my_node = Node {
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                Equation {
                    scope: Scope::Output,
                    id: String::from("o"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("*"),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer, Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![
                            StreamExpression::SignalCall {
                                signal: Signal {
                                    id: String::from("x"),
                                    scope: Scope::Input,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                            },
                            StreamExpression::SignalCall {
                                signal: Signal {
                                    id: String::from("y"),
                                    scope: Scope::Input,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                            },
                        ],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![
                            (String::from("x"), 0),
                            (String::from("y"), 0),
                        ]),
                    },
                    location: Location::default(),
                },
            )]),
            unitary_nodes: HashMap::from([(
                String::from("o"),
                UnitaryNode {
                    node_id: String::from("my_node"),
                    output_id: String::from("o"),
                    inputs: vec![
                        (String::from("x"), Type::Integer),
                        (String::from("y"), Type::Integer),
                    ],
                    equations: vec![Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("*"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("x"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                                },
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("y"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![
                                (String::from("x"), 0),
                                (String::from("y"), 0),
                            ]),
                        },
                        location: Location::default(),
                    }],
                    memory: Memory::new(),
                    location: Location::default(),
                    graph: OnceCell::new(),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::from(my_node_graph),
        };
        // other_node(x: int, y: int) { out o1: int = x+y; out o2: int = 2*y; }
        let other_node = Node {
            id: String::from("other_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("+"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("x"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                                },
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("y"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![
                                (String::from("x"), 0),
                                (String::from("y"), 0),
                            ]),
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
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("*"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::Constant {
                                    constant: Constant::Integer(2),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![]),
                                },
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("y"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::from([
                (
                    String::from("o1"),
                    UnitaryNode {
                        node_id: String::from("other_node"),
                        output_id: String::from("o1"),
                        inputs: vec![
                            (String::from("x"), Type::Integer),
                            (String::from("y"), Type::Integer),
                        ],
                        equations: vec![Equation {
                            scope: Scope::Output,
                            id: String::from("o1"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: String::from("+"),
                                    typing: Some(Type::Abstract(
                                        vec![Type::Integer, Type::Integer],
                                        Box::new(Type::Integer),
                                    )),
                                    location: Location::default(),
                                },
                                inputs: vec![
                                    StreamExpression::SignalCall {
                                        signal: Signal {
                                            id: String::from("x"),
                                            scope: Scope::Input,
                                        },
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::from(vec![(
                                            String::from("x"),
                                            0,
                                        )]),
                                    },
                                    StreamExpression::SignalCall {
                                        signal: Signal {
                                            id: String::from("y"),
                                            scope: Scope::Input,
                                        },
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::from(vec![(
                                            String::from("y"),
                                            0,
                                        )]),
                                    },
                                ],
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![
                                    (String::from("x"), 0),
                                    (String::from("y"), 0),
                                ]),
                            },
                            location: Location::default(),
                        }],
                        memory: Memory::new(),
                        location: Location::default(),
                        graph: OnceCell::new(),
                    },
                ),
                (
                    String::from("o2"),
                    UnitaryNode {
                        node_id: String::from("other_node"),
                        output_id: String::from("o2"),
                        inputs: vec![(String::from("y"), Type::Integer)],
                        equations: vec![Equation {
                            scope: Scope::Output,
                            id: String::from("o2"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: String::from("*"),
                                    typing: Some(Type::Abstract(
                                        vec![Type::Integer, Type::Integer],
                                        Box::new(Type::Integer),
                                    )),
                                    location: Location::default(),
                                },
                                inputs: vec![
                                    StreamExpression::Constant {
                                        constant: Constant::Integer(2),
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::from(vec![]),
                                    },
                                    StreamExpression::SignalCall {
                                        signal: Signal {
                                            id: String::from("y"),
                                            scope: Scope::Input,
                                        },
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::from(vec![(
                                            String::from("y"),
                                            0,
                                        )]),
                                    },
                                ],
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                            },
                            location: Location::default(),
                        }],
                        memory: Memory::new(),
                        location: Location::default(),
                        graph: OnceCell::new(),
                    },
                ),
            ]),
            location: Location::default(),
            graph: OnceCell::from(other_node_graph),
        };

        // out x: int = 1 + my_node(s, v*2).o
        let unitary_equation_1 = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: String::from("+"),
                    typing: Some(Type::Abstract(
                        vec![Type::Integer, Type::Integer],
                        Box::new(Type::Integer),
                    )),
                    location: Location::default(),
                },
                inputs: vec![
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![]),
                    },
                    StreamExpression::UnitaryNodeApplication {
                        id: None,
                        node: String::from("my_node"),
                        inputs: vec![
                            (
                                format!("x"),
                                StreamExpression::SignalCall {
                                    signal: Signal {
                                        id: String::from("s"),
                                        scope: Scope::Input,
                                    },
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                                },
                            ),
                            (
                                format!("y"),
                                StreamExpression::MapApplication {
                                    function_expression: Expression::Call {
                                        id: String::from("*2"),
                                        typing: Some(Type::Abstract(
                                            vec![Type::Integer],
                                            Box::new(Type::Integer),
                                        )),
                                        location: Location::default(),
                                    },
                                    inputs: vec![StreamExpression::SignalCall {
                                        signal: Signal {
                                            id: String::from("v"),
                                            scope: Scope::Input,
                                        },
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::from(vec![(
                                            String::from("v"),
                                            0,
                                        )]),
                                    }],
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                                },
                            ),
                        ],
                        signal: String::from("o"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![
                            (String::from("s"), 0),
                            (String::from("v"), 0),
                        ]),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("s"), 0),
                    (String::from("v"), 0),
                ]),
            },
            location: Location::default(),
        };
        let unitary_node_1 = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            equations: vec![unitary_equation_1],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        // out y: int = other_node(g-1, v).o1
        let unitary_equation_2 = Equation {
            scope: Scope::Output,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                id: None,
                node: String::from("other_node"),
                inputs: vec![
                    (
                        format!("x"),
                        StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("-1"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![StreamExpression::SignalCall {
                                signal: Signal {
                                    id: String::from("g"),
                                    scope: Scope::Input,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("g"), 0)]),
                            }],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("g"), 0)]),
                        },
                    ),
                    (
                        format!("y"),
                        StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("v"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                        },
                    ),
                ],
                signal: String::from("o1"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("g"), 0),
                    (String::from("v"), 0),
                ]),
            },
            location: Location::default(),
        };
        let unitary_node_2 = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("y"),
            inputs: vec![
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            equations: vec![unitary_equation_2],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        // out z: int = other_node(g-1, v).o2
        let unitary_equation_3 = Equation {
            scope: Scope::Output,
            id: String::from("z"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                id: None,
                node: String::from("other_node"),
                inputs: vec![(
                    format!("y"),
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("v"),
                            scope: Scope::Input,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                    },
                )],
                signal: String::from("o2"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
            },
            location: Location::default(),
        };
        let unitary_node_3 = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("z"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![unitary_equation_3],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1),
                (String::from("y"), equation_2),
                (String::from("z"), equation_3),
            ]),
            unitary_nodes: HashMap::from([
                (String::from("x"), unitary_node_1),
                (String::from("y"), unitary_node_2),
                (String::from("z"), unitary_node_3),
            ]),
            location: Location::default(),
            graph: OnceCell::from(node_graph),
        };

        let function = Function {
            id: String::from("my_function"),
            inputs: vec![(String::from("i"), Type::Integer)],
            statements: vec![Statement {
                id: String::from("x"),
                element_type: Type::Integer,
                expression: Expression::Call {
                    id: String::from("i"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                location: Location::default(),
            }],
            returned: (
                Type::Integer,
                Expression::Call {
                    id: String::from("x"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };
        let control = File {
            typedefs: vec![],
            functions: vec![function],
            nodes: vec![my_node, other_node, node],
            component: None,
            location: Location::default(),
        };
        assert_eq!(
            file.nodes
                .get(2)
                .unwrap()
                .unscheduled_equations
                .get(&String::from("z"))
                .unwrap(),
            control
                .nodes
                .get(2)
                .unwrap()
                .unscheduled_equations
                .get(&String::from("z"))
                .unwrap()
        );
        assert_eq!(file, control);
    }
}
