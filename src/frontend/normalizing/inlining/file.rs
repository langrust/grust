use std::collections::HashMap;

use crate::hir::file::File;

impl File {
    /// Inline node application when it is needed.
    ///
    /// Inlining needed for "shifted causality loop".
    ///
    /// # Example:
    /// ```GR
    /// node semi_fib(i: int) {
    ///     out o: int = 0 fby (i + 1 fby i);
    /// }
    /// node fib_call() {
    ///    out fib: int = semi_fib(fib).o;
    /// }
    /// ```
    /// In this example, `fib_call` calls `semi_fib` with the same input and output signal.
    /// There is no causality loop, `o` depends on the memory of `i`.
    ///
    /// We need to inline the code, the output `fib` is defined before the input `fib`,
    /// which can not be computed by a function call.
    pub fn inline_when_needed(&mut self) {
        let nodes = self
            .nodes
            .iter()
            .map(|node| (node.id.clone(), node.clone()))
            .collect::<HashMap<_, _>>();
        self.nodes
            .iter_mut()
            .for_each(|node| node.inline_when_needed(&nodes));
        self.component
            .as_mut()
            .map_or((), |component| component.inline_when_needed(&nodes))
    }
}

#[cfg(test)]
mod inline_when_needed {

    use crate::ast::expression::Expression;
    use crate::common::graph::color::Color;
    use crate::common::graph::Graph;
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, file::File, memory::Memory, node::Node,
        once_cell::OnceCell, signal::Signal, stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    };
    use std::collections::HashMap;

    #[test]
    fn should_inline_root_node_calls_when_inputs_depends_on_outputs() {
        // node my_node(i: int, j: int) {
        //     out o: int = i + (0 fby j);
        // }
        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::Black);
        graph.add_vertex(String::from("i"), Color::Black);
        graph.add_vertex(String::from("j"), Color::Black);
        graph.add_edge(&String::from("o"), String::from("i"), 0);
        graph.add_edge(&String::from("o"), String::from("j"), 1);
        let my_node_equation = Equation {
            scope: Scope::Output,
            id: String::from("o"),
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
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("i"),
                            scope: Scope::Input,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                    },
                    StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("j"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("j"), 0)]),
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("j"), 1)]),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("i"), 0),
                    (String::from("j"), 1),
                ]),
            },
            location: Location::default(),
        };
        let my_node =Node { assertions: Default::default(), contracts: (vec![], vec![]),
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("i"), Type::Integer),
                (String::from("j"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(String::from("o"), my_node_equation.clone())]),
            unitary_nodes: HashMap::from([(
                String::from("o"),
                UnitaryNode { contracts: (vec![], vec![]),
                    node_id: String::from("my_node"),
                    output_id: String::from("o"),
                    inputs: vec![
                        (String::from("i"), Type::Integer),
                        (String::from("j"), Type::Integer),
                    ],
                    equations: vec![my_node_equation],
                    memory: Memory::new(),
                    location: Location::default(),
                    graph: OnceCell::from(graph.clone()),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::from(graph.clone()),
        };

        // node other_node(i: int) {
        //     out o: int = 0 fby i;
        // }
        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::Black);
        graph.add_vertex(String::from("i"), Color::Black);
        graph.add_edge(&String::from("o"), String::from("i"), 1);
        let other_node_equation = Equation {
            scope: Scope::Output,
            id: String::from("o"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
                constant: Constant::Integer(0),
                expression: Box::new(StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("i"),
                        scope: Scope::Input,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                }),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("i"), 1)]),
            },
            location: Location::default(),
        };
        let other_node =Node { assertions: Default::default(), contracts: (vec![], vec![]),
            id: String::from("other_node"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                other_node_equation.clone(),
            )]),
            unitary_nodes: HashMap::from([(
                String::from("o"),
                UnitaryNode { contracts: (vec![], vec![]),
                    node_id: String::from("other_node"),
                    output_id: String::from("o"),
                    inputs: vec![(String::from("i"), Type::Integer)],
                    equations: vec![other_node_equation],
                    memory: Memory::new(),
                    location: Location::default(),
                    graph: OnceCell::from(graph.clone()),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::from(graph.clone()),
        };

        // x: int = my_node(v*2, x).o
        let equation_1 = Equation {
            scope: Scope::Local,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                id: Some(format!("my_node_o_x")),
                node: String::from("my_node"),
                inputs: vec![
                    (
                        format!("i"),
                        StreamExpression::FunctionApplication {
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
                    ),
                    (
                        format!("j"),
                        StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("x"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        },
                    ),
                ],
                signal: String::from("o"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("v"), 0),
                    (String::from("x"), 1),
                ]),
            },
            location: Location::default(),
        };
        // out y: int = other_node(x-1).o
        let equation_2 = Equation {
            scope: Scope::Output,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                id: Some(format!("other_node_o_y")),
                node: String::from("other_node"),
                inputs: vec![(
                    format!("i"),
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
                                id: String::from("x"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                )],
                signal: String::from("o"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("x"), 1)]),
            },
            location: Location::default(),
        };
        // node test(v: int) {
        //     x: int = my_node(v*2, x).o
        //     out y: int = other_node(x-1).o
        // }
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("y"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("v"), 0);
        graph.add_edge(&String::from("x"), String::from("x"), 1);
        graph.add_edge(&String::from("y"), String::from("x"), 1);
        let node =Node { assertions: Default::default(), contracts: (vec![], vec![]),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1.clone()),
                (String::from("y"), equation_2.clone()),
            ]),
            unitary_nodes: HashMap::from([(
                String::from("y"),
                UnitaryNode { contracts: (vec![], vec![]),
                    node_id: String::from("test"),
                    output_id: String::from("y"),
                    inputs: vec![(String::from("v"), Type::Integer)],
                    equations: vec![equation_1.clone(), equation_2.clone()],
                    memory: Memory::new(),
                    location: Location::default(),
                    graph: OnceCell::from(graph.clone()),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::from(graph.clone()),
        };

        let mut file = File {
            typedefs: vec![],
            functions: vec![],
            nodes: vec![my_node, other_node, node],
            component: None,
            location: Location::default(),
        };

        file.inline_when_needed();
        let node = file.nodes.get(2).unwrap();

        // x: int = v*2 + 0 fby x
        let inlined_equation = Equation {
            scope: Scope::Local,
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
                    StreamExpression::FunctionApplication {
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
                    StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("x"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 1)]),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("v"), 0),
                    (String::from("x"), 1),
                ]),
            },
            location: Location::default(),
        };
        // out y: int = other_node(x-1).o
        let new_equation_2 = Equation {
            scope: Scope::Output,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                id: Some(format!("other_node_o_y")),
                node: String::from("other_node"),
                inputs: vec![(
                    format!("i"),
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
                                id: String::from("x"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                )],
                signal: String::from("o"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
            },
            location: Location::default(),
        };

        let mut unitary_graph = Graph::new();
        unitary_graph.add_vertex(String::from("v"), Color::White);
        unitary_graph.add_vertex(String::from("x"), Color::White);
        unitary_graph.add_vertex(String::from("y"), Color::White);
        unitary_graph.add_edge(&String::from("x"), String::from("v"), 0);
        unitary_graph.add_edge(&String::from("x"), String::from("x"), 1);
        unitary_graph.add_edge(&String::from("y"), String::from("x"), 0);

        // node test(v: int) {
        //     x: int = v*2 + 0 fby x
        //     out y: int = other_node(x-1).o
        // }
        let control =Node { assertions: Default::default(), contracts: (vec![], vec![]),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1.clone()),
                (String::from("y"), equation_2.clone()),
            ]),
            unitary_nodes: HashMap::from([(
                String::from("y"),
                UnitaryNode { contracts: (vec![], vec![]),
                    node_id: String::from("test"),
                    output_id: String::from("y"),
                    inputs: vec![(String::from("v"), Type::Integer)],
                    equations: vec![inlined_equation, new_equation_2],
                    memory: Memory::new(),
                    location: Location::default(),
                    graph: OnceCell::from(unitary_graph),
                },
            )]),
            location: Location::default(),

            graph: OnceCell::from(graph),
        };

        assert_eq!(node, &control)
    }

    #[test]
    fn should_inline_all_node_calls_when_inputs_depends_on_outputs() {
        // node my_node(i: int, j: int) {
        //     out o: int = i + (0 fby j);
        // }
        let mut graph = Graph::new();
        graph.add_vertex(format!("i"), Color::White);
        graph.add_vertex(format!("j"), Color::White);
        graph.add_vertex(format!("o"), Color::White);
        graph.add_edge(&format!("o"), format!("i"), 0);
        graph.add_edge(&format!("o"), format!("j"), 1);
        let my_node_equation = Equation {
            scope: Scope::Output,
            id: String::from("o"),
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
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("i"),
                            scope: Scope::Input,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                    },
                    StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("j"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("j"), 0)]),
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("j"), 1)]),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("i"), 0),
                    (String::from("j"), 1),
                ]),
            },
            location: Location::default(),
        };
        let my_node =Node { assertions: Default::default(), contracts: (vec![], vec![]),
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("i"), Type::Integer),
                (String::from("j"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(String::from("o"), my_node_equation.clone())]),
            unitary_nodes: HashMap::from([(
                String::from("o"),
                UnitaryNode { contracts: (vec![], vec![]),
                    node_id: String::from("my_node"),
                    output_id: String::from("o"),
                    inputs: vec![
                        (String::from("i"), Type::Integer),
                        (String::from("j"), Type::Integer),
                    ],
                    equations: vec![my_node_equation],
                    memory: Memory::new(),
                    location: Location::default(),
                    graph: OnceCell::from(graph.clone()),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::from(graph.clone()),
        };

        // node other_node(i: int) {
        //     out o: int = 0 fby i;
        // }
        let mut graph = Graph::new();
        graph.add_vertex(format!("i"), Color::White);
        graph.add_vertex(format!("o"), Color::White);
        graph.add_edge(&format!("o"), format!("i"), 1);
        let other_node_equation = Equation {
            scope: Scope::Output,
            id: String::from("o"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
                constant: Constant::Integer(0),
                expression: Box::new(StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("i"),
                        scope: Scope::Input,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                }),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("i"), 1)]),
            },
            location: Location::default(),
        };
        let other_node =Node { assertions: Default::default(), contracts: (vec![], vec![]),
            id: String::from("other_node"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                other_node_equation.clone(),
            )]),
            unitary_nodes: HashMap::from([(
                String::from("o"),
                UnitaryNode { contracts: (vec![], vec![]),
                    node_id: String::from("other_node"),
                    output_id: String::from("o"),
                    inputs: vec![(String::from("i"), Type::Integer)],
                    equations: vec![other_node_equation],
                    memory: Memory::new(),
                    location: Location::default(),
                    graph: OnceCell::from(graph.clone()),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::from(graph),
        };

        // x: int = 1 + my_node(v*2, x).o
        let equation_1 = Equation {
            scope: Scope::Local,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::FunctionApplication {
                function_expression: Expression::Call {
                    id: String::from("1+"),
                    typing: Some(Type::Abstract(
                        vec![Type::Integer, Type::Integer],
                        Box::new(Type::Integer),
                    )),
                    location: Location::default(),
                },
                inputs: vec![StreamExpression::UnitaryNodeApplication {
                    id: Some(format!("my_node_o_x")),
                    node: String::from("my_node"),
                    inputs: vec![
                        (
                            format!("i"),
                            StreamExpression::FunctionApplication {
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
                        ),
                        (
                            format!("j"),
                            StreamExpression::SignalCall {
                                signal: Signal {
                                    id: String::from("x"),
                                    scope: Scope::Input,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                            },
                        ),
                    ],
                    signal: String::from("o"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![
                        (String::from("v"), 0),
                        (String::from("x"), 1),
                    ]),
                }],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("v"), 0),
                    (String::from("x"), 1),
                ]),
            },
            location: Location::default(),
        };
        // out y: int = other_node(x-1).o
        let equation_2 = Equation {
            scope: Scope::Output,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                id: Some(format!("other_node_o_y")),
                node: String::from("other_node"),
                inputs: vec![(
                    format!("i"),
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
                                id: String::from("x"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                )],
                signal: String::from("o"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("x"), 1)]),
            },
            location: Location::default(),
        };
        // node test(v: int) {
        //     x: int = 1 + my_node(v*2, x).o
        //     out y: int = other_node(x-1).o
        // }
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("y"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("v"), 0);
        graph.add_edge(&String::from("x"), String::from("x"), 1);
        graph.add_edge(&String::from("y"), String::from("x"), 1);
        let node =Node { assertions: Default::default(), contracts: (vec![], vec![]),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1.clone()),
                (String::from("y"), equation_2.clone()),
            ]),
            unitary_nodes: HashMap::from([(
                String::from("y"),
                UnitaryNode { contracts: (vec![], vec![]),
                    node_id: String::from("test"),
                    output_id: String::from("y"),
                    inputs: vec![(String::from("v"), Type::Integer)],
                    equations: vec![equation_1.clone(), equation_2.clone()],
                    memory: Memory::new(),
                    location: Location::default(),
                    graph: OnceCell::from(graph.clone()),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::from(graph.clone()),
        };

        let mut file = File {
            typedefs: vec![],
            functions: vec![],
            nodes: vec![my_node, other_node, node],
            component: None,
            location: Location::default(),
        };

        file.inline_when_needed();
        let node = file.nodes.get(2).unwrap();

        // o: int = v*2 + 0 fby x
        let added_equation = Equation {
            scope: Scope::Local,
            id: String::from("o"),
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
                    StreamExpression::FunctionApplication {
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
                    StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("x"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 1)]),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("v"), 0),
                    (String::from("x"), 1),
                ]),
            },
            location: Location::default(),
        };
        // x: int = 1 + o
        let inlined_equation = Equation {
            scope: Scope::Local,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::FunctionApplication {
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
                        id: String::from("o"),
                        scope: Scope::Local,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                }],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
            },
            location: Location::default(),
        };
        // out y: int = other_node(x-1).o
        let new_equation_2 = Equation {
            scope: Scope::Output,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                id: Some(format!("other_node_o_y")),
                node: String::from("other_node"),
                inputs: vec![(
                    format!("i"),
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
                                id: String::from("x"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                )],
                signal: String::from("o"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
            },
            location: Location::default(),
        };

        let mut unitary_graph = Graph::new();
        unitary_graph.add_vertex(String::from("v"), Color::White);
        unitary_graph.add_vertex(String::from("x"), Color::White);
        unitary_graph.add_vertex(String::from("y"), Color::White);
        unitary_graph.add_vertex(String::from("o"), Color::White);
        unitary_graph.add_edge(&String::from("x"), String::from("o"), 0);
        unitary_graph.add_edge(&String::from("o"), String::from("v"), 0);
        unitary_graph.add_edge(&String::from("o"), String::from("x"), 1);
        unitary_graph.add_edge(&String::from("y"), String::from("x"), 0);

        // node test(v: int) {
        //     o: int = v*2 + 0 fby x
        //     x: int = 1 + o
        //     out y: int = other_node(x-1).o
        // }
        let control =Node { assertions: Default::default(), contracts: (vec![], vec![]),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1.clone()),
                (String::from("y"), equation_2.clone()),
            ]),
            unitary_nodes: HashMap::from([(
                String::from("y"),
                UnitaryNode { contracts: (vec![], vec![]),
                    node_id: String::from("test"),
                    output_id: String::from("y"),
                    inputs: vec![(String::from("v"), Type::Integer)],
                    equations: vec![added_equation, inlined_equation, new_equation_2],
                    memory: Memory::new(),
                    location: Location::default(),
                    graph: OnceCell::from(unitary_graph),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::from(graph),
        };

        assert_eq!(node.unitary_nodes, control.unitary_nodes)
    }

    #[test]
    fn should_inline_node_calls_when_needed_recursively() {
        // node my_node(i: int, j: int) {
        //     out o: int = i + other_node(j).o;
        // }
        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::Black);
        graph.add_vertex(String::from("i"), Color::Black);
        graph.add_vertex(String::from("j"), Color::Black);
        graph.add_edge(&String::from("o"), String::from("i"), 0);
        graph.add_edge(&String::from("o"), String::from("j"), 1);
        let my_node_equation = Equation {
            scope: Scope::Output,
            id: String::from("o"),
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
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("i"),
                            scope: Scope::Input,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                    },
                    StreamExpression::UnitaryNodeApplication {
                        id: Some(format!("other_node_o_o")),
                        node: String::from("other_node"),
                        signal: String::from("o"),
                        inputs: vec![(
                            format!("i"),
                            StreamExpression::SignalCall {
                                signal: Signal {
                                    id: String::from("j"),
                                    scope: Scope::Input,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("j"), 0)]),
                            },
                        )],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("j"), 1)]),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("i"), 0),
                    (String::from("j"), 1),
                ]),
            },
            location: Location::default(),
        };
        let my_node =Node { assertions: Default::default(), contracts: (vec![], vec![]),
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("i"), Type::Integer),
                (String::from("j"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(String::from("o"), my_node_equation.clone())]),
            unitary_nodes: HashMap::from([(
                String::from("o"),
                UnitaryNode { contracts: (vec![], vec![]),
                    node_id: String::from("my_node"),
                    output_id: String::from("o"),
                    inputs: vec![
                        (String::from("i"), Type::Integer),
                        (String::from("j"), Type::Integer),
                    ],
                    equations: vec![my_node_equation],
                    memory: Memory::new(),
                    location: Location::default(),
                    graph: OnceCell::from(graph.clone()),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::from(graph.clone()),
        };

        // node other_node(i: int) {
        //     out o: int = 0 fby i;
        // }
        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::Black);
        graph.add_vertex(String::from("i"), Color::Black);
        graph.add_edge(&String::from("o"), String::from("i"), 1);
        let other_node_equation = Equation {
            scope: Scope::Output,
            id: String::from("o"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
                constant: Constant::Integer(0),
                expression: Box::new(StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("i"),
                        scope: Scope::Input,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                }),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("i"), 1)]),
            },
            location: Location::default(),
        };
        let other_node =Node { assertions: Default::default(), contracts: (vec![], vec![]),
            id: String::from("other_node"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                other_node_equation.clone(),
            )]),
            unitary_nodes: HashMap::from([(
                String::from("o"),
                UnitaryNode { contracts: (vec![], vec![]),
                    node_id: String::from("other_node"),
                    output_id: String::from("o"),
                    inputs: vec![(String::from("i"), Type::Integer)],
                    equations: vec![other_node_equation],
                    memory: Memory::new(),
                    location: Location::default(),
                    graph: OnceCell::from(graph.clone()),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::from(graph.clone()),
        };

        // x: int = my_node(v*2, x).o
        let equation_1 = Equation {
            scope: Scope::Local,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                id: Some(format!("my_node_o_x")),
                node: String::from("my_node"),
                inputs: vec![
                    (
                        format!("i"),
                        StreamExpression::FunctionApplication {
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
                    ),
                    (
                        format!("j"),
                        StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("x"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        },
                    ),
                ],
                signal: String::from("o"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("v"), 0),
                    (String::from("x"), 1),
                ]),
            },
            location: Location::default(),
        };
        // out y: int = other_node(x-1).o
        let equation_2 = Equation {
            scope: Scope::Output,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                id: Some(format!("other_node_o_y")),
                node: String::from("other_node"),
                inputs: vec![(
                    format!("i"),
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
                                id: String::from("x"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                )],
                signal: String::from("o"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("x"), 1)]),
            },
            location: Location::default(),
        };
        // node test(v: int) {
        //     x: int = my_node(v*2, x).o
        //     out y: int = other_node(x-1).o
        // }
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("y"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("v"), 0);
        graph.add_edge(&String::from("x"), String::from("x"), 1);
        graph.add_edge(&String::from("y"), String::from("x"), 1);
        let node =Node { assertions: Default::default(), contracts: (vec![], vec![]),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1.clone()),
                (String::from("y"), equation_2.clone()),
            ]),
            unitary_nodes: HashMap::from([(
                String::from("y"),
                UnitaryNode { contracts: (vec![], vec![]),
                    node_id: String::from("test"),
                    output_id: String::from("y"),
                    inputs: vec![(String::from("v"), Type::Integer)],
                    equations: vec![equation_1.clone(), equation_2.clone()],
                    memory: Memory::new(),
                    location: Location::default(),
                    graph: OnceCell::from(graph.clone()),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::from(graph.clone()),
        };

        let mut file = File {
            typedefs: vec![],
            functions: vec![],
            nodes: vec![my_node, other_node, node],
            component: None,
            location: Location::default(),
        };

        file.inline_when_needed();
        let node = file.nodes.get(2).unwrap();

        // o_1: int = 0 fby x
        let inlined_equation = Equation {
            scope: Scope::Local,
            id: String::from("o_1"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
                constant: Constant::Integer(0),
                expression: Box::new(StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("x"),
                        scope: Scope::Input,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                }),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("x"), 1)]),
            },
            location: Location::default(),
        };
        // x: int = v*2 + o_1
        let new_equation_1 = Equation {
            scope: Scope::Local,
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
                    StreamExpression::FunctionApplication {
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
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("o_1"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("o_1"), 0)]),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![
                    (String::from("v"), 0),
                    (String::from("o_1"), 0),
                ]),
            },
            location: Location::default(),
        };
        // out y: int = other_node(x-1).o
        let new_equation_2 = Equation {
            scope: Scope::Output,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                id: Some(format!("other_node_o_y")),
                node: String::from("other_node"),
                inputs: vec![(
                    format!("i"),
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
                                id: String::from("x"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                    },
                )],
                signal: String::from("o"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
            },
            location: Location::default(),
        };

        let mut unitary_graph = Graph::new();
        unitary_graph.add_vertex(String::from("v"), Color::White);
        unitary_graph.add_vertex(String::from("x"), Color::White);
        unitary_graph.add_vertex(String::from("y"), Color::White);
        unitary_graph.add_vertex(String::from("o_1"), Color::White);
        unitary_graph.add_edge(&String::from("x"), String::from("v"), 0);
        unitary_graph.add_edge(&String::from("x"), String::from("o_1"), 0);
        unitary_graph.add_edge(&String::from("o_1"), String::from("x"), 1);
        unitary_graph.add_edge(&String::from("y"), String::from("x"), 0);

        // node test(v: int) {
        //     o_1: int = 0 fby x
        //     x: int = v*2 + o_1
        //     out y: int = other_node(x-1).o
        // }
        let control =Node { assertions: Default::default(), contracts: (vec![], vec![]),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1.clone()),
                (String::from("y"), equation_2.clone()),
            ]),
            unitary_nodes: HashMap::from([(
                String::from("y"),
                UnitaryNode { contracts: (vec![], vec![]),
                    node_id: String::from("test"),
                    output_id: String::from("y"),
                    inputs: vec![(String::from("v"), Type::Integer)],
                    equations: vec![inlined_equation, new_equation_1, new_equation_2],
                    memory: Memory::new(),
                    location: Location::default(),
                    graph: OnceCell::from(unitary_graph),
                },
            )]),
            location: Location::default(),
            graph: OnceCell::from(graph.clone()),
        };

        assert_eq!(node, &control)
    }
}
