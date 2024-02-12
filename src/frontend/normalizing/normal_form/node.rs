use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::{common::graph::neighbor::Label, hir::node::Node};

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
    pub fn normal_form(
        &mut self,
        nodes_reduced_graphs: &HashMap<String, DiGraphMap<String, Label>>,
    ) {
        self.unitary_nodes
            .values_mut()
            .for_each(|unitary_node| unitary_node.normal_form(nodes_reduced_graphs))
    }
}

#[cfg(test)]
mod normal_form {
    use std::collections::HashMap;

    use petgraph::graphmap::GraphMap;

    use crate::ast::expression::Expression;
    use crate::common::graph::neighbor::Label;
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, memory::Memory, node::Node,
        once_cell::OnceCell, signal::Signal, stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    };

    #[test]
    fn should_change_node_applications_to_be_root_expressions() {
        let mut graph = GraphMap::new();
        graph.add_node(String::from("x"));
        graph.add_node(String::from("y"));
        graph.add_node(String::from("o"));
        graph.add_edge(String::from("o"), String::from("x"), Label::Weight(0));
        graph.add_edge(String::from("o"), String::from("y"), Label::Weight(0));
        let nodes_reduced_graphs = HashMap::from([(format!("my_node"), graph)]);

        // node test(s: int, v: int) {
        //     out x: int = 1 + my_node(s, v).o;
        // }
        let equation = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::FunctionApplication {
                function_expression: Expression::Identifier {
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
            contract: Default::default(),
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
            contract: Default::default(),
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
        node.normal_form(&nodes_reduced_graphs);

        // node test(s: int, v: int) {
        //     x_1: int = my_node(s, v).o;
        //     out x: int = 1 + x_1;
        // }
        let mut graph = GraphMap::new();
        graph.add_node(String::from("x_1"));
        graph.add_node(String::from("x"));
        graph.add_node(String::from("s"));
        graph.add_node(String::from("v"));
        graph.add_edge(String::from("x_1"), String::from("s"), Label::Weight(0));
        graph.add_edge(String::from("x_1"), String::from("v"), Label::Weight(0));
        graph.add_edge(String::from("x"), String::from("x_1"), Label::Weight(0));
        let equations = vec![
            Equation {
                scope: Scope::Local,
                id: String::from("x_1"),
                signal_type: Type::Integer,
                expression: StreamExpression::UnitaryNodeApplication {
                    id: Some(format!("my_node_o_x_1")),
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
                expression: StreamExpression::FunctionApplication {
                    function_expression: Expression::Identifier {
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
                                id: String::from("x_1"),
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
            contract: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            equations: equations,
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::from(graph),
        };
        let control = Node {
            contract: Default::default(),
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
        let mut graph = GraphMap::new();
        graph.add_node(String::from("x"));
        graph.add_node(String::from("y"));
        graph.add_node(String::from("o"));
        graph.add_edge(String::from("o"), String::from("x"), Label::Weight(0));
        graph.add_edge(String::from("o"), String::from("y"), Label::Weight(0));
        let nodes_reduced_graphs = HashMap::from([(format!("other_node"), graph)]);

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
                        StreamExpression::FunctionApplication {
                            function_expression: Expression::Identifier {
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
            contract: Default::default(),
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
            contract: Default::default(),
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
        node.normal_form(&nodes_reduced_graphs);

        // node test(v: int, g: int) {
        //     x: int = g-1;
        //     out y: int = other_node(x, v).o;
        // }
        let mut graph = GraphMap::new();
        graph.add_node(String::from("x"));
        graph.add_node(String::from("y"));
        graph.add_node(String::from("g"));
        graph.add_node(String::from("v"));
        graph.add_edge(String::from("y"), String::from("x"), Label::Weight(0));
        graph.add_edge(String::from("y"), String::from("v"), Label::Weight(0));
        graph.add_edge(String::from("x"), String::from("g"), Label::Weight(0));
        let equations = vec![
            Equation {
                scope: Scope::Local,
                id: String::from("x"),
                signal_type: Type::Integer,
                expression: StreamExpression::FunctionApplication {
                    function_expression: Expression::Identifier {
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
                    id: Some(format!("other_node_o_y")),
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
            contract: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("y"),
            inputs: vec![
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            equations: equations,
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::from(graph),
        };
        let control = Node {
            contract: Default::default(),
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
        let mut graph = GraphMap::new();
        graph.add_node(String::from("x"));
        graph.add_node(String::from("y"));
        graph.add_node(String::from("o"));
        graph.add_edge(String::from("o"), String::from("x"), Label::Weight(0));
        graph.add_edge(String::from("o"), String::from("y"), Label::Weight(0));
        let nodes_reduced_graphs = HashMap::from([(format!("other_node"), graph)]);

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
                        StreamExpression::FunctionApplication {
                            function_expression: Expression::Identifier {
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
            contract: Default::default(),
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
            contract: Default::default(),
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
        node.normal_form(&nodes_reduced_graphs);

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
