use crate::hir::node::Node;

impl Node {
    /// Create memory for HIR node's unitary nodes.
    ///
    /// Store buffer for followed by expressions and unitary node applications.
    /// Transform followed by expressions in signal call.
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     x_1: int = 0 fby v;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     x_1: int = mem;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// memory test {
    ///     buffers: {
    ///         mem: int = 0 fby v;
    ///     },
    ///     called_nodes: {
    ///         memmy_nodeo: (my_node, o);
    ///     },
    /// }
    /// ```
    ///
    /// This example is tested in source.
    pub fn memorize(&mut self) {
        self.unitary_nodes
            .values_mut()
            .for_each(|unitary_node| unitary_node.memorize())
    }
}

#[cfg(test)]
mod memorize {

    use std::collections::HashMap;

    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, memory::Memory, node::Node,
        once_cell::OnceCell, signal::Signal, stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    };

    #[test]
    fn should_memorize_followed_by() {
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
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("s"),
                            scope: Scope::Input,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                    },
                    StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("v"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("v"), 1)]),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("s"), 0),
                    (String::from("v"), 1),
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
            unitary_nodes: HashMap::from([(String::from("o"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        node.memorize();

        let new_equation = Equation {
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
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("s"),
                            scope: Scope::Input,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                    },
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("mem"),
                            scope: Scope::Memory,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("mem"), 0)]),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("s"), 0),
                    (String::from("v"), 1),
                ]),
            },
            location: Location::default(),
        };
        let mut memory = Memory::new();
        memory.add_buffer(
            String::from("mem"),
            Constant::Integer(0),
            StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("v"),
                    scope: Scope::Input,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
            },
        );
        let control = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(String::from("x"), equation)]),
            unitary_nodes: HashMap::from([(
                String::from("o"),
                UnitaryNode {
                    node_id: String::from("test"),
                    output_id: String::from("x"),
                    inputs: vec![
                        (String::from("s"), Type::Integer),
                        (String::from("v"), Type::Integer),
                    ],
                    equations: vec![new_equation],
                    memory,
                    location: Location::default(),
                    graph: OnceCell::new(),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        assert_eq!(node, control);
    }

    #[test]
    fn should_memorize_node_expression() {
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
                    id: Some(format!("my_nodeoy")),
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
                                id: String::from("x_2"),
                                scope: Scope::Local,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x_2"), 0)]),
                        },
                    ],
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
            equations: equations.clone(),
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
            unscheduled_equations: equations
                .clone()
                .into_iter()
                .map(|equation| (equation.id.clone(), equation))
                .collect(),
            unitary_nodes: HashMap::from([(String::from("x"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        node.memorize();

        let mut memory = Memory::new();
        memory.add_called_node(
            String::from("my_nodeoy"),
            String::from("my_node"),
            String::from("o"),
        );
        let control = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            unscheduled_equations: equations
                .clone()
                .into_iter()
                .map(|equation| (equation.id.clone(), equation))
                .collect(),
            unitary_nodes: HashMap::from([(
                String::from("x"),
                UnitaryNode {
                    node_id: String::from("test"),
                    output_id: String::from("x"),
                    inputs: vec![
                        (String::from("s"), Type::Integer),
                        (String::from("v"), Type::Integer),
                    ],
                    equations: equations,
                    memory,
                    location: Location::default(),
                    graph: OnceCell::new(),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        assert_eq!(node, control);
    }
}
