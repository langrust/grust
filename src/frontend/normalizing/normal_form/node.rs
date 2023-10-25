use crate::hir::node::Node;

impl Node {
    /// Change HIR node into a normal form.
    ///
    /// The normal form of a node is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
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
    pub fn normal_form(&mut self) {
        self.unitary_nodes
            .values_mut()
            .for_each(|unitary_node| unitary_node.normal_form())
    }
}

#[cfg(test)]
mod normal_form {
    use once_cell::sync::OnceCell;
    use std::collections::HashMap;

    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, memory::Memory, node::Node, signal::Signal,
        stream_expression::StreamExpression, unitary_node::UnitaryNode,
    };

    #[test]
    fn should_change_node_applications_to_be_root_expressions() {
        // node test(s: int, v: int) {
        //     out x: int = 1 + my_node(s, v).o;
        // }
        let equation = Equation {
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
        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            equations: vec![equation.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(String::from("x"), equation.clone())]),
            unitary_nodes: HashMap::from([(String::from("x"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        node.normal_form();

        // node test(s: int, v: int) {
        //     x_1: int = my_node(s, v).o;
        //     out x: int = 1 + x_1;
        // }
        let equations = vec![
            Equation {
                scope: Scope::Local,
                id: String::from("x_1"),
                signal_type: Type::Integer,
                expression: StreamExpression::UnitaryNodeApplication {
                    id: Some(format!("my_nodeox_1")),
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
                                    id: String::from("v"),
                                    scope: Scope::Input,
                                },
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
                location: Location::default(),
            },
            Equation {
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
                        StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("1"),
                                scope: Scope::Local,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x_1"), 0)]),
                        },
                    ],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("x_1"), 0)]),
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
            graph: OnceCell::new(),
        };
        let control = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(String::from("x"), equation)]),
            unitary_nodes: HashMap::from([(String::from("x"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        assert_eq!(node, control);
    }

    #[test]
    fn should_change_inputs_expressions_to_be_signal_calls() {
        // node test(v: int, g: int) {
        //     out y: int = other_node(g-1, v).o;
        // }
        let equation = Equation {
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
                signal: String::from("o"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("g"), 0),
                    (String::from("v"), 0),
                ]),
            },
            location: Location::default(),
        };
        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("y"),
            inputs: vec![
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            equations: vec![equation.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(String::from("y"), equation.clone())]),
            unitary_nodes: HashMap::from([(String::from("y"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        node.normal_form();

        // node test(v: int, g: int) {
        //     x: int = g-1;
        //     out y: int = other_node(x, v).o;
        // }
        let equations = vec![
            Equation {
                scope: Scope::Local,
                id: String::from("x"),
                signal_type: Type::Integer,
                expression: StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("-1"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
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
                location: Location::default(),
            },
            Equation {
                scope: Scope::Output,
                id: String::from("y"),
                signal_type: Type::Integer,
                expression: StreamExpression::UnitaryNodeApplication {
                    id: Some(format!("other_nodeoy")),
                    node: String::from("other_node"),
                    inputs: vec![
                        (
                            format!("x"),
                            StreamExpression::SignalCall {
                                signal: Signal {
                                    id: String::from("x"),
                                    scope: Scope::Local,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
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
                    signal: String::from("o"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![
                        (String::from("x"), 0),
                        (String::from("v"), 0),
                    ]),
                },
                location: Location::default(),
            },
        ];
        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("y"),
            inputs: vec![
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            equations: equations,
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let control = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(String::from("y"), equation)]),
            unitary_nodes: HashMap::from([(String::from("y"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        assert_eq!(node, control);
    }

    #[test]
    fn should_set_identifier_to_node_state_in_unitary_node_application() {
        // node test(v: int, g: int) {
        //     out y: int = other_node(g-1, v).o;
        // }
        let equation = Equation {
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
                signal: String::from("o"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("g"), 0),
                    (String::from("v"), 0),
                ]),
            },
            location: Location::default(),
        };
        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("y"),
            inputs: vec![
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            equations: vec![equation.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(String::from("y"), equation.clone())]),
            unitary_nodes: HashMap::from([(String::from("y"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        node.normal_form();

        for Equation { expression, .. } in node
            .unitary_nodes
            .get(&String::from("y"))
            .unwrap()
            .equations
            .iter()
        {
            if let StreamExpression::UnitaryNodeApplication { id, .. } = expression {
                assert!(id.is_some())
            }
        }
    }
}
