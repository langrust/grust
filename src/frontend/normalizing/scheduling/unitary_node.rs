use crate::hir::unitary_node::UnitaryNode;

impl UnitaryNode {
    /// Schedule equations.
    ///
    /// # Example.
    ///
    /// ```GR
    /// node test(v: int) {
    ///     out y: int = x-1
    ///     o_1: int = 0 fby x
    ///     x: int = v*2 + o_1
    /// }
    /// ```
    ///
    /// In the node above, signal `y` depends on the current value of `x`,
    /// `o_1` depends on the memory of `x` and `x` depends on `v` and `o_1`.
    /// The node is causal and should be scheduled as bellow:
    ///
    /// ```GR
    /// node test(v: int) {
    ///     o_1: int = 0 fby x  // depends on no current values of signals
    ///     x: int = v*2 + o_1  // depends on the computed value of `o_1` and given `v`
    ///     out y: int = x-1    // depends on the computed value of `x`
    /// }
    /// ```
    pub fn schedule(&mut self) {
        let mut subgraph = self
            .graph
            .get_mut()
            .unwrap()
            .subgraph_on_edges(|weight| weight == 0);

        let schedule = subgraph.topological_sorting().unwrap();

        let scheduled_equations = schedule
            .into_iter()
            .filter_map(|signal_id| {
                self.equations
                    .iter()
                    .position(|equation| equation.id == signal_id)
            })
            .map(|index| self.equations.get(index).unwrap().clone())
            .collect();

        self.equations = scheduled_equations;
    }
}

#[cfg(test)]
mod schedule {
    use crate::ast::expression::Expression;
    use crate::common::graph::color::Color;
    use crate::common::graph::Graph;
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, memory::Memory, once_cell::OnceCell,
        signal::Signal, stream_expression::StreamExpression, unitary_node::UnitaryNode,
    };

    #[test]
    fn should_schedule_equations_according_to_dependencies() {
        // out y: int = other_node(x-1).o
        let equation_1 = Equation {
            scope: Scope::Output,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                id: None,
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
                                scope: Scope::Local,
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
        // o_1: int = 0 fby x
        let equation_2 = Equation {
            scope: Scope::Local,
            id: String::from("o_1"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
                constant: Constant::Integer(0),
                expression: Box::new(StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("x"),
                        scope: Scope::Local,
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
        let equation_3 = Equation {
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
                            id: String::from("x_1"),
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
        // node test(v: int) {
        //     out y: int = other_node(x-1).o
        //     o_1: int = 0 fby x
        //     x: int = v*2 + o_1
        // }
        let mut unitary_node = UnitaryNode { contracts: (vec![], vec![]),
            node_id: String::from("test"),
            output_id: String::from("y"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_1.clone(), equation_2.clone(), equation_3.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v"), Color::Black);
        graph.add_vertex(String::from("y"), Color::Black);
        graph.add_vertex(String::from("o_1"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_edge(&String::from("y"), String::from("x"), 0);
        graph.add_edge(&String::from("o_1"), String::from("x"), 1);
        graph.add_edge(&String::from("x"), String::from("v"), 0);
        graph.add_edge(&String::from("x"), String::from("o_1"), 0);
        unitary_node.graph.set(graph.clone()).unwrap();

        unitary_node.schedule();

        // node test(v: int) {
        //     o_1: int = 0 fby x
        //     x: int = v*2 + o_1
        //     out y: int = other_node(x-1).o
        // }
        let control = UnitaryNode { contracts: (vec![], vec![]),
            node_id: String::from("test"),
            output_id: String::from("y"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_2.clone(), equation_3.clone(), equation_1.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        control.graph.set(graph).unwrap();

        assert_eq!(unitary_node, control)
    }

    #[test]
    fn should_ensure_unscheduled_equality() {
        // out y: int = other_node(x-1).o
        let equation_1 = Equation {
            scope: Scope::Output,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                id: None,
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
                                scope: Scope::Local,
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
        // o_1: int = 0 fby x
        let equation_2 = Equation {
            scope: Scope::Local,
            id: String::from("o_1"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
                constant: Constant::Integer(0),
                expression: Box::new(StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("x"),
                        scope: Scope::Local,
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
        let equation_3 = Equation {
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
                            id: String::from("x_1"),
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
        // node test(v: int) {
        //     out y: int = other_node(x-1).o
        //     o_1: int = 0 fby x
        //     x: int = v*2 + o_1
        // }
        let mut unitary_node = UnitaryNode { contracts: (vec![], vec![]),
            node_id: String::from("test"),
            output_id: String::from("y"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_1.clone(), equation_2.clone(), equation_3.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut graph = Graph::new();
        graph.add_vertex(String::from("v"), Color::Black);
        graph.add_vertex(String::from("y"), Color::Black);
        graph.add_vertex(String::from("o_1"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_edge(&String::from("y"), String::from("x"), 0);
        graph.add_edge(&String::from("o_1"), String::from("x"), 1);
        graph.add_edge(&String::from("x"), String::from("v"), 0);
        graph.add_edge(&String::from("x"), String::from("o_1"), 0);
        unitary_node.graph.set(graph).unwrap();

        let control = unitary_node.clone();

        unitary_node.schedule();

        assert!(unitary_node.eq_unscheduled(&control))
    }
}
