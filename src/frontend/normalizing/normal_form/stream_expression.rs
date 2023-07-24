use crate::common::scope::Scope;
use crate::hir::{
    dependencies::Dependencies, equation::Equation, identifier_creator::IdentifierCreator,
    stream_expression::StreamExpression,
};

impl StreamExpression {
    /// Normalize HIR expressions.
    ///
    /// Normalize HIR expressions as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// x: int = 1 + my_node(s, v*2).o;
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// x_1: int = v*2;
    /// x_2: int = my_node(s, x_1).o;
    /// x: int = 1 + x_2;
    /// ```
    pub fn normalize(&mut self, identifier_creator: &mut IdentifierCreator) -> Vec<Equation> {
        self.normalize_root(identifier_creator)
    }

    fn normalize_root(&mut self, identifier_creator: &mut IdentifierCreator) -> Vec<Equation> {
        match self {
            StreamExpression::FollowedBy { expression, .. } => {
                expression.normalize_cascade(identifier_creator)
            }
            StreamExpression::MapApplication { inputs, .. } => inputs
                .iter_mut()
                .flat_map(|expression| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::NodeApplication { .. } => unreachable!(),
            StreamExpression::UnitaryNodeApplication { inputs, .. } => {
                let equations = inputs
                    .iter_mut()
                    .flat_map(|expression| expression.normalize_to_signal_call(identifier_creator))
                    .collect();

                equations
            }
            StreamExpression::Structure { fields, .. } => fields
                .iter_mut()
                .flat_map(|(_, expression)| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::Array { elements, .. } => elements
                .iter_mut()
                .flat_map(|expression| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::Match {
                expression, arms, ..
            } => {
                let mut equations = expression.normalize_cascade(identifier_creator);
                arms.iter_mut().for_each(|(_, option, body, expression)| {
                    let mut option_equations = option.as_mut().map_or(vec![], |option| {
                        option.normalize_cascade(identifier_creator)
                    });
                    equations.append(&mut option_equations);

                    let mut expression_equations = expression.normalize_cascade(identifier_creator);
                    body.append(&mut expression_equations)
                });
                equations
            }
            StreamExpression::When {
                option,
                present_body,
                present,
                default_body,
                default,
                ..
            } => {
                let mut present_equations = present.normalize_cascade(identifier_creator);
                present_body.append(&mut present_equations);

                let mut default_equations = default.normalize_cascade(identifier_creator);
                default_body.append(&mut default_equations);

                option.normalize_cascade(identifier_creator)
            }
            _ => vec![],
        }
    }

    fn normalize_cascade(&mut self, identifier_creator: &mut IdentifierCreator) -> Vec<Equation> {
        match self {
            StreamExpression::FollowedBy { expression, .. } => {
                expression.normalize_cascade(identifier_creator)
            }
            StreamExpression::MapApplication { inputs, .. } => inputs
                .iter_mut()
                .flat_map(|expression| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::Structure { fields, .. } => fields
                .iter_mut()
                .flat_map(|(_, expression)| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::Array { elements, .. } => elements
                .iter_mut()
                .flat_map(|expression| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::Match {
                expression, arms, ..
            } => {
                let mut equations = expression.normalize_cascade(identifier_creator);
                arms.iter_mut().for_each(|(_, option, body, expression)| {
                    let mut option_equations = option.as_mut().map_or(vec![], |option| {
                        option.normalize_cascade(identifier_creator)
                    });
                    equations.append(&mut option_equations);

                    let mut expression_equations = expression.normalize_cascade(identifier_creator);
                    body.append(&mut expression_equations)
                });
                equations
            }
            StreamExpression::When {
                option,
                present_body,
                present,
                default_body,
                default,
                ..
            } => {
                let mut present_equations = present.normalize_cascade(identifier_creator);
                present_body.append(&mut present_equations);

                let mut default_equations = default.normalize_cascade(identifier_creator);
                default_body.append(&mut default_equations);

                option.normalize_cascade(identifier_creator)
            }
            StreamExpression::NodeApplication { .. } => unreachable!(),
            StreamExpression::UnitaryNodeApplication { inputs, .. } => {
                let mut equations = inputs
                    .iter_mut()
                    .flat_map(|expression| expression.normalize_to_signal_call(identifier_creator))
                    .collect::<Vec<_>>();

                let fresh_id = identifier_creator.new_identifier(
                    String::from("x"),
                    String::from(""),
                    String::from(""),
                );

                let typing = self.get_type().clone();
                let location = self.get_location().clone();

                let unitary_node_application_equation = Equation {
                    scope: Scope::Local,
                    signal_type: typing.clone(),
                    location: location.clone(),
                    expression: self.clone(),
                    id: fresh_id.clone(),
                };

                *self = StreamExpression::SignalCall {
                    id: fresh_id.clone(),
                    typing: typing,
                    location: location,
                    dependencies: Dependencies::from(vec![(fresh_id, 0)]),
                };

                equations.push(unitary_node_application_equation);

                equations
            }
            _ => vec![],
        }
    }

    fn normalize_to_signal_call(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
    ) -> Vec<Equation> {
        match self {
            StreamExpression::SignalCall { .. } => vec![],
            _ => {
                let mut equations = self.normalize_cascade(identifier_creator);

                let typing = self.get_type().clone();
                let location = self.get_location().clone();

                let fresh_id = identifier_creator.new_identifier(
                    String::from("x"),
                    String::from(""),
                    String::from(""),
                );

                let new_equation = Equation {
                    scope: Scope::Local,
                    signal_type: typing.clone(),
                    location: location.clone(),
                    expression: self.clone(),
                    id: fresh_id.clone(),
                };

                *self = StreamExpression::SignalCall {
                    id: fresh_id.clone(),
                    typing: typing,
                    location: location,
                    dependencies: Dependencies::from(vec![(fresh_id, 0)]),
                };

                equations.push(new_equation);
                equations
            }
        }
    }
}

#[cfg(test)]
mod normalize_to_signal_call {
    use std::collections::HashSet;

    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, identifier_creator::IdentifierCreator,
        stream_expression::StreamExpression,
    };

    #[test]
    fn should_leave_signal_call_unchanged() {
        let mut identifier_creator = IdentifierCreator {
            signals: HashSet::new(),
        };

        let mut expression = StreamExpression::SignalCall {
            id: String::from("x"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
        };
        let equations = expression.normalize_to_signal_call(&mut identifier_creator);

        let control = StreamExpression::SignalCall {
            id: String::from("x"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
        };
        assert!(equations.is_empty());
        assert_eq!(expression, control)
    }

    #[test]
    fn should_create_signal_call_from_other_expression() {
        let mut identifier_creator = IdentifierCreator {
            signals: HashSet::from([String::from("x")]),
        };

        let mut expression = StreamExpression::FollowedBy {
            constant: Constant::Integer(0),
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
            }),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("x"), 1)]),
        };
        let equations = expression.normalize_to_signal_call(&mut identifier_creator);

        let control = Equation {
            scope: Scope::Local,
            id: String::from("x_1"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
                constant: Constant::Integer(0),
                expression: Box::new(StreamExpression::SignalCall {
                    id: String::from("x"),
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
        assert_eq!(equations[0], control);

        let control = StreamExpression::SignalCall {
            id: String::from("x_1"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("x_1"), 0)]),
        };
        assert_eq!(expression, control)
    }
}

#[cfg(test)]
mod normalize {
    use std::collections::HashSet;

    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, identifier_creator::IdentifierCreator,
        stream_expression::StreamExpression,
    };

    #[test]
    fn should_normalize_node_applications_to_be_root_expressions() {
        // x: int = 1 + my_node(s, v*2).o;
        let mut identifier_creator = IdentifierCreator {
            signals: HashSet::from([String::from("x"), String::from("s"), String::from("v")]),
        };

        let mut expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("+"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
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
                    node: String::from("my_node"),
                    inputs: vec![
                        StreamExpression::SignalCall {
                            id: String::from("s"),
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
                                id: String::from("v"),
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
            dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 0)]),
        };
        let equations = expression.normalize(&mut identifier_creator);

        // x_2: int = my_node(s, x_1).o;
        let control = Equation {
            scope: Scope::Local,
            id: String::from("x_2"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                node: String::from("my_node"),
                inputs: vec![
                    StreamExpression::SignalCall {
                        id: String::from("s"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                    },
                    StreamExpression::SignalCall {
                        id: String::from("x_1"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x_1"), 0)]),
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
            location: Location::default(),
        };
        assert_eq!(*equations.get(1).unwrap(), control);

        // x: int = 1 + x_2;
        let control = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("+"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
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
                    id: String::from("x_2"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("x_2"), 0)]),
                },
            ],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 0)]),
        };
        assert_eq!(expression, control)
    }

    #[test]
    fn should_normalize_inputs_expressions_to_be_signal_calls() {
        // x: int = 1 + my_node(s, v*2).o;
        let mut identifier_creator = IdentifierCreator {
            signals: HashSet::from([String::from("x"), String::from("s"), String::from("v")]),
        };

        let mut expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("+"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
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
                    node: String::from("my_node"),
                    inputs: vec![
                        StreamExpression::SignalCall {
                            id: String::from("s"),
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
                                id: String::from("v"),
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
            dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 0)]),
        };
        let equations = expression.normalize(&mut identifier_creator);

        // x_1: int = v*2;
        // x_2: int = my_node(s, x_1).o;
        let control = vec![
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
                        id: String::from("v"),
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
                    node: String::from("my_node"),
                    inputs: vec![
                        StreamExpression::SignalCall {
                            id: String::from("s"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                        },
                        StreamExpression::SignalCall {
                            id: String::from("x_1"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x_1"), 0)]),
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
                location: Location::default(),
            },
        ];
        assert_eq!(equations, control)
    }
}
