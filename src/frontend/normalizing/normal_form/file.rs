use crate::hir::file::File;

impl File {
    /// Change HIR file into a normal form.
    ///
    /// The normal form of a node is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// function test(i: int) -> int {
    ///     let x: int = i;
    ///     return x;
    /// }
    /// node my_node(x: int, y: int) {
    ///     out o: int = x*y;
    /// }
    /// node other_node(x: int, y: int) {
    ///     out o: int = x*y;
    /// }
    /// node test(s: int, v: int, g: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    ///     out y: int = other_node(g-1, v).o;
    /// }
    /// ```
    ///
    /// The above node contains the following unitary nodes:
    ///
    /// ```GR
    /// node test_x(s: int, v: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    /// }
    /// node test_y(v: int, g: int) {
    ///     out y: int = other_node(g-1, v).o;
    /// }
    /// ```
    ///
    /// Which are transformed into:
    ///
    /// ```GR
    /// node test_x(s: int, v: int) {
    ///     x_1: int = v*2;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// node test_y(v: int, g: int) {
    ///     x: int = g-1;
    ///     out y: int = other_node(x_1, v).o;
    /// }
    /// ```
    ///
    /// This example is tested in source.
    pub fn normal_form(&mut self) {
        self.nodes.iter_mut().for_each(|node| node.normal_form());
        self.component
            .as_mut()
            .map_or((), |component| component.normal_form())
    }
}

#[cfg(test)]
mod normal_form {
    use crate::ast::{expression::Expression, function::Function, statement::Statement};
    use crate::common::graph::color::Color;
    use crate::common::graph::Graph;
    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, file::File, memory::Memory, node::Node,
        once_cell::OnceCell, signal::Signal, stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    };
    use std::collections::HashMap;

    #[test]
    fn should_change_nodes_to_their_normal_form_in_file() {
        // node my_node(x: int, y: int) {
        //     out o: int = x * y
        // }
        let mut my_node_graph = Graph::new();
        my_node_graph.add_vertex(String::from("x"), Color::Black);
        my_node_graph.add_vertex(String::from("y"), Color::Black);
        my_node_graph.add_vertex(String::from("o"), Color::Black);
        my_node_graph.add_edge(&String::from("o"), String::from("x"), 0);
        my_node_graph.add_edge(&String::from("o"), String::from("y"), 0);
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
                    graph: OnceCell::from(my_node_graph.clone()),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::from(my_node_graph.clone()),
        };

        // node test(x: int, y: int) {
        //     out x: int = 1 + my_node(s, v*2).o
        // }
        let mut node_graph = Graph::new();
        node_graph.add_vertex(String::from("s"), Color::Black);
        node_graph.add_vertex(String::from("v"), Color::Black);
        node_graph.add_vertex(String::from("g"), Color::Black);
        node_graph.add_vertex(String::from("x"), Color::Black);
        node_graph.add_vertex(String::from("y"), Color::Black);
        node_graph.add_edge(&String::from("x"), String::from("s"), 0);
        node_graph.add_edge(&String::from("x"), String::from("v"), 0);
        node_graph.add_edge(&String::from("y"), String::from("g"), 0);
        node_graph.add_edge(&String::from("y"), String::from("v"), 0);
        let node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(
                String::from("x"),
                Equation {
                    scope: Scope::Output,
                    id: String::from("x"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("1+"),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer, Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::NodeApplication {
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
                                        dependencies: Dependencies::from(vec![(
                                            String::from("v"),
                                            0,
                                        )]),
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
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![
                            (String::from("s"), 0),
                            (String::from("v"), 0),
                        ]),
                    },
                    location: Location::default(),
                },
            )]),
            unitary_nodes: HashMap::from([(
                String::from("x"),
                UnitaryNode {
                    node_id: String::from("test"),
                    output_id: String::from("x"),
                    inputs: vec![
                        (String::from("s"), Type::Integer),
                        (String::from("v"), Type::Integer),
                    ],
                    equations: vec![Equation {
                        scope: Scope::Output,
                        id: String::from("x"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("1+"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![StreamExpression::UnitaryNodeApplication {
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
                                            dependencies: Dependencies::from(vec![(
                                                String::from("s"),
                                                0,
                                            )]),
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
                                            dependencies: Dependencies::from(vec![(
                                                String::from("v"),
                                                0,
                                            )]),
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
                            }],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![
                                (String::from("s"), 0),
                                (String::from("v"), 0),
                            ]),
                        },
                        location: Location::default(),
                    }],
                    memory: Memory::new(),
                    location: Location::default(),
                    graph: OnceCell::from(node_graph.clone()),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::from(node_graph.clone()),
        };
        let function = Function {
            id: String::from("test"),
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
            nodes: vec![my_node, node],
            component: None,
            location: Location::default(),
        };
        file.normal_form();

        // node my_node(x: int, y: int) {
        //     out o: int = x * y
        // }
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
                    graph: OnceCell::from(my_node_graph.clone()),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::from(my_node_graph),
        };

        // node test(x: int, y: int) {
        //     x_1: int = v*2
        //     x_2: int = my_node(s, x_1).o
        //     out x: int = 1 + x_2
        // }
        let equations = vec![
            Equation {
                scope: Scope::Local,
                id: String::from("x_1"),
                signal_type: Type::Integer,
                expression: StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("*2"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
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
                location: Location::default(),
            },
            Equation {
                scope: Scope::Local,
                id: String::from("x_2"),
                signal_type: Type::Integer,
                expression: StreamExpression::UnitaryNodeApplication {
                    id: Some(format!("my_nodeox_2")),
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
                            StreamExpression::SignalCall {
                                signal: Signal {
                                    id: String::from("x_1"),
                                    scope: Scope::Local,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("x_1"), 0)]),
                            },
                        ),
                    ],
                    signal: String::from("o"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![
                        (String::from("s"), 0),
                        (String::from("x_1"), 0),
                    ]),
                },
                location: Location::default(),
            },
            Equation {
                scope: Scope::Output,
                id: String::from("x"),
                signal_type: Type::Integer,
                expression: StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("1+"),
                        typing: Some(Type::Abstract(
                            vec![Type::Integer, Type::Integer],
                            Box::new(Type::Integer),
                        )),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x_2"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x_2"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("x_2"), 0)]),
                },
                location: Location::default(),
            },
        ];
        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            equations: equations,
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::from(node_graph.clone()),
        };
        let node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(
                String::from("x"),
                Equation {
                    scope: Scope::Output,
                    id: String::from("x"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("1+"),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer, Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::NodeApplication {
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
                                        dependencies: Dependencies::from(vec![(
                                            String::from("v"),
                                            0,
                                        )]),
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
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![
                            (String::from("s"), 0),
                            (String::from("v"), 0),
                        ]),
                    },
                    location: Location::default(),
                },
            )]),
            unitary_nodes: HashMap::from([(String::from("x"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::from(node_graph),
        };
        let function = Function {
            id: String::from("test"),
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
            nodes: vec![my_node, node],
            component: None,
            location: Location::default(),
        };
        assert_eq!(file, control);
    }
}
