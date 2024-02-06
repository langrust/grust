use std::collections::HashMap;

use crate::{
    common::graph::{color::Color, Graph},
    hir::{equation::Equation, identifier_creator::IdentifierCreator, unitary_node::UnitaryNode},
};

impl UnitaryNode {
    /// Change HIR unitary node into a normal form.
    ///
    /// The normal form of an unitary node is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    /// }
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// node test(s: int, v: int) {
    ///     x_1: int = v*2;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// ```
    pub fn normal_form(&mut self, nodes_reduced_graphs: &HashMap<String, Graph<Color>>) {
        let mut identifier_creator = IdentifierCreator::from(self.get_signals());

        let UnitaryNode { equations, .. } = self;

        *equations = equations
            .clone()
            .into_iter()
            .flat_map(|equation| {
                equation.normal_form(nodes_reduced_graphs, &mut identifier_creator)
            })
            .collect();

        // add a dependency graph to the unitary node
        let mut graph = Graph::new();
        self.get_signals()
            .iter()
            .for_each(|signal_id| graph.add_vertex(signal_id.clone(), Color::White));
        self.equations.iter().for_each(
            |Equation {
                 id: from,
                 expression,
                 ..
             }| {
                for (to, weight) in expression.get_dependencies() {
                    graph.add_weighted_edge(from, to.clone(), *weight)
                }
            },
        );
        self.graph.set(graph).unwrap();
    }
}

#[cfg(test)]
mod normal_form {

    use std::collections::HashMap;

    use crate::ast::expression::Expression;
    use crate::common::graph::color::Color;
    use crate::common::graph::Graph;
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, memory::Memory, once_cell::OnceCell,
        signal::Signal, stream_expression::StreamExpression, unitary_node::UnitaryNode,
    };

    #[test]
    fn should_change_node_applications_to_be_root_expressions() {
        let mut graph = Graph::new();
        graph.add_vertex(format!("x"), Color::White);
        graph.add_vertex(format!("y"), Color::White);
        graph.add_vertex(format!("o"), Color::White);
        graph.add_weighted_edge(&format!("o"), format!("x"), 0);
        graph.add_weighted_edge(&format!("o"), format!("y"), 0);
        let nodes_reduced_graphs = HashMap::from([(format!("my_node"), graph)]);

        // node test(s: int, v: int) {
        //     out x: int = 1 + my_node(s, v).o;
        // }
        let equation = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::FunctionApplication {
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
        let mut unitary_node = UnitaryNode {
            contracts: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            equations: vec![equation],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        unitary_node.normal_form(&nodes_reduced_graphs);

        // node test(s: int, v: int) {
        //     x_1: int = my_node(s, v).o;
        //     out x: int = 1 + x_1;
        // }
        let mut graph = Graph::new();
        graph.add_vertex(format!("x_1"), Color::White);
        graph.add_vertex(format!("x"), Color::White);
        graph.add_vertex(format!("s"), Color::White);
        graph.add_vertex(format!("v"), Color::White);
        graph.add_weighted_edge(&format!("x_1"), format!("s"), 0);
        graph.add_weighted_edge(&format!("x_1"), format!("v"), 0);
        graph.add_weighted_edge(&format!("x"), format!("x_1"), 0);
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
        let control = UnitaryNode {
            contracts: Default::default(),
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
        assert_eq!(unitary_node, control);
    }

    #[test]
    fn should_change_inputs_expressions_to_be_signal_calls() {
        let mut graph = Graph::new();
        graph.add_vertex(format!("x"), Color::White);
        graph.add_vertex(format!("y"), Color::White);
        graph.add_vertex(format!("o"), Color::White);
        graph.add_weighted_edge(&format!("o"), format!("x"), 0);
        graph.add_weighted_edge(&format!("o"), format!("y"), 0);
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
        let mut unitary_node = UnitaryNode {
            contracts: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("y"),
            inputs: vec![
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            equations: vec![equation],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        unitary_node.normal_form(&nodes_reduced_graphs);

        // node test(v: int, g: int) {
        //     x: int = g-1;
        //     out y: int = other_node(x, v).o;
        // }
        let mut graph = Graph::new();
        graph.add_vertex(format!("x"), Color::White);
        graph.add_vertex(format!("y"), Color::White);
        graph.add_vertex(format!("g"), Color::White);
        graph.add_vertex(format!("v"), Color::White);
        graph.add_weighted_edge(&format!("y"), format!("x"), 0);
        graph.add_weighted_edge(&format!("y"), format!("v"), 0);
        graph.add_weighted_edge(&format!("x"), format!("g"), 0);
        let equations = vec![
            Equation {
                scope: Scope::Local,
                id: String::from("x"),
                signal_type: Type::Integer,
                expression: StreamExpression::FunctionApplication {
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
        let control = UnitaryNode {
            contracts: Default::default(),
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
        assert_eq!(unitary_node, control);
    }

    #[test]
    fn should_set_identifier_to_node_state_in_unitary_node_application() {
        let mut graph = Graph::new();
        graph.add_vertex(format!("x"), Color::White);
        graph.add_vertex(format!("y"), Color::White);
        graph.add_vertex(format!("o"), Color::White);
        graph.add_weighted_edge(&format!("o"), format!("x"), 0);
        graph.add_weighted_edge(&format!("o"), format!("y"), 0);
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
        let mut unitary_node = UnitaryNode {
            contracts: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("y"),
            inputs: vec![
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            equations: vec![equation],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        unitary_node.normal_form(&nodes_reduced_graphs);

        for Equation { expression, .. } in unitary_node.equations {
            if let StreamExpression::UnitaryNodeApplication { id, .. } = expression {
                assert!(id.is_some())
            }
        }
    }
}
