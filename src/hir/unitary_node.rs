use once_cell::sync::OnceCell;

use crate::common::{
    graph::{color::Color, Graph},
    location::Location,
    r#type::Type,
};
use crate::hir::{equation::Equation, identifier_creator::IdentifierCreator, memory::Memory};

#[derive(Debug, PartialEq, Clone)]
/// LanGRust unitary node HIR.
pub struct UnitaryNode {
    /// Mother node identifier.
    pub node_id: String,
    /// Output signal identifier.
    pub output_id: String,
    /// Unitary node's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Unitary node's equations.
    pub equations: Vec<Equation>,
    /// Unitary node's memory.
    pub memory: Memory,
    /// Mother node location.
    pub location: Location,
    /// Unitary node dependency graph.
    pub graph: OnceCell<Graph<Color>>,
}

impl UnitaryNode {
    /// Return vector of unitary node's signals.
    pub fn get_signals(&self) -> Vec<String> {
        let mut signals = vec![];
        self.inputs.iter().for_each(|(signal, _)| {
            signals.push(signal.clone());
        });
        self.equations.iter().for_each(|equation| {
            signals.push(equation.id.clone());
        });
        signals
    }

    /// Tells if two unscheduled unitary nodes are equal.
    pub fn eq_unscheduled(&self, other: &UnitaryNode) -> bool {
        self.node_id == other.node_id
            && self.output_id == other.output_id
            && self.inputs == other.inputs
            && self.equations.len() == other.equations.len()
            && self.equations.iter().all(|equation| {
                other
                    .equations
                    .iter()
                    .any(|other_equation| equation == other_equation)
            })
            && self.memory == other.memory
            && self.location == other.location
    }

    /// Create memory for HIR unitary nodes.
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
        let mut identifier_creator = IdentifierCreator::from(self.get_signals());
        let mut memory = Memory::new();

        self.equations
            .iter_mut()
            .for_each(|equation| equation.memorize(&mut identifier_creator, &mut memory));

        self.memory = memory;
    }
}

#[cfg(test)]
mod get_signals {
    use once_cell::sync::OnceCell;

    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, memory::Memory, signal::Signal,
        stream_expression::StreamExpression, unitary_node::UnitaryNode,
    };

    #[test]
    fn should_return_all_signals_from_unitary_node() {
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
            equations: vec![equation],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let mut signals = unitary_node.get_signals();

        let mut control = vec![String::from("x"), String::from("s"), String::from("v")];
        assert_eq!(signals.len(), control.len());
        while let Some(id) = signals.pop() {
            let index = control.iter().position(|r| r.eq(&id)).unwrap();
            let _ = control.remove(index);
        }
    }
}

#[cfg(test)]
mod eq_unscheduled {
    use once_cell::sync::OnceCell;

    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, memory::Memory, signal::Signal,
        stream_expression::StreamExpression, unitary_node::UnitaryNode,
    };

    #[test]
    fn should_return_true_for_strictly_equal_unitary_nodes() {
        let equation_1 = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("y"),
                    scope: Scope::Local,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
            },
            location: Location::default(),
        };
        let equation_2 = Equation {
            scope: Scope::Local,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
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
            location: Location::default(),
        };
        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_1, equation_2],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let other = unitary_node.clone();

        assert!(unitary_node.eq_unscheduled(&other))
    }

    #[test]
    fn should_return_true_for_strictly_equal_unitary_nodes_unscheduled() {
        let equation_1 = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("y"),
                    scope: Scope::Local,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
            },
            location: Location::default(),
        };
        let equation_2 = Equation {
            scope: Scope::Local,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
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
            location: Location::default(),
        };
        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_1.clone(), equation_2.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let other = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_2, equation_1],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        assert!(unitary_node.eq_unscheduled(&other))
    }

    #[test]
    fn should_return_false_for_missing_equations() {
        let equation_1 = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("y"),
                    scope: Scope::Local,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
            },
            location: Location::default(),
        };
        let equation_2 = Equation {
            scope: Scope::Local,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
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
            location: Location::default(),
        };
        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_1.clone(), equation_2],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let other = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_1.clone(), equation_1],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        assert!(!unitary_node.eq_unscheduled(&other))
    }

    #[test]
    fn should_return_false_for_too_much_equations() {
        let equation_1 = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("y"),
                    scope: Scope::Local,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
            },
            location: Location::default(),
        };
        let equation_2 = Equation {
            scope: Scope::Local,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
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
            location: Location::default(),
        };
        let unitary_node = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_1.clone(), equation_2.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let other = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_1.clone(), equation_2.clone(), equation_1],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        assert!(!unitary_node.eq_unscheduled(&other))
    }
}

#[cfg(test)]
mod memorize {
    use once_cell::sync::OnceCell;

    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, memory::Memory, signal::Signal,
        stream_expression::StreamExpression, unitary_node::UnitaryNode,
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
        let mut unitary_node = UnitaryNode {
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
        unitary_node.memorize();

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
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("m"),
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
        let control = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            equations: vec![equation],
            memory,
            location: Location::default(),
            graph: OnceCell::new(),
        };
        assert_eq!(unitary_node, control);
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
                                    id: String::from("1"),
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
                                id: String::from("2"),
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
        let mut unitary_node = UnitaryNode {
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
        unitary_node.memorize();

        let mut memory = Memory::new();
        memory.add_called_node(
            String::from("my_nodeoy"),
            String::from("my_node"),
            String::from("o"),
        );
        let control = UnitaryNode {
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
        };
        assert_eq!(unitary_node, control);
    }
}
