use crate::common::scope::Scope;
use crate::hir::{
    dependencies::Dependencies, equation::Equation, identifier_creator::IdentifierCreator,
    stream_expression::StreamExpression,
};

impl StreamExpression {
    /// Change HIR expression into a normal form.
    ///
    /// The normal form of an expression is as follows:
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
    pub fn normal_form(&mut self, identifier_creator: &mut IdentifierCreator) -> Vec<Equation> {
        self.normal_form_root(identifier_creator)
    }

    fn normal_form_root(&mut self, identifier_creator: &mut IdentifierCreator) -> Vec<Equation> {
        match self {
            StreamExpression::FollowedBy {
                expression,
                ref mut dependencies,
                ..
            } => {
                let new_equations = expression.normal_form_cascade(identifier_creator);

                *dependencies = Dependencies::from(
                    expression
                        .get_dependencies()
                        .iter()
                        .map(|(id, depth)| (id.clone(), *depth + 1))
                        .collect(),
                );

                new_equations
            }
            StreamExpression::MapApplication {
                inputs,
                ref mut dependencies,
                ..
            } => {
                let new_equations = inputs
                    .iter_mut()
                    .flat_map(|expression| expression.normal_form_cascade(identifier_creator))
                    .collect();

                *dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|expression| expression.get_dependencies().clone())
                        .collect(),
                );

                new_equations
            }
            StreamExpression::NodeApplication { .. } => unreachable!(),
            StreamExpression::UnitaryNodeApplication {
                id: None,
                inputs,
                ref mut dependencies,
                ..
            } => {
                let new_equations = inputs
                    .iter_mut()
                    .flat_map(|(_, expression)| {
                        expression.normal_form_to_signal_call(identifier_creator)
                    })
                    .collect();

                // change dependencies to be the sum of inputs dependencies
                *dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|(_, expression)| expression.get_dependencies().clone())
                        .collect(),
                );

                new_equations
            }
            StreamExpression::Structure {
                fields,
                ref mut dependencies,
                ..
            } => {
                let new_equations = fields
                    .iter_mut()
                    .flat_map(|(_, expression)| expression.normal_form_cascade(identifier_creator))
                    .collect();

                *dependencies = Dependencies::from(
                    fields
                        .iter()
                        .flat_map(|(_, expression)| expression.get_dependencies().clone())
                        .collect(),
                );

                new_equations
            }
            StreamExpression::Array {
                elements,
                ref mut dependencies,
                ..
            } => {
                let new_equations = elements
                    .iter_mut()
                    .flat_map(|expression| expression.normal_form_cascade(identifier_creator))
                    .collect();

                *dependencies = Dependencies::from(
                    elements
                        .iter()
                        .flat_map(|expression| expression.get_dependencies().clone())
                        .collect(),
                );

                new_equations
            }
            StreamExpression::Match {
                expression,
                arms,
                ref mut dependencies,
                ..
            } => {
                let mut equations = expression.normal_form_cascade(identifier_creator);
                let mut expression_dependencies = expression.get_dependencies().clone();

                arms.iter_mut()
                    .for_each(|(_, bound, body, matched_expression)| {
                        let (mut bound_equations, mut bound_dependencies) =
                            bound.as_mut().map_or((vec![], vec![]), |expression| {
                                (
                                    expression.normal_form_cascade(identifier_creator),
                                    expression.get_dependencies().clone(),
                                )
                            });
                        equations.append(&mut bound_equations);
                        expression_dependencies.append(&mut bound_dependencies);

                        let mut matched_expression_equations =
                            matched_expression.normal_form_cascade(identifier_creator);
                        let mut matched_expression_dependencies =
                            matched_expression.get_dependencies().clone();
                        body.append(&mut matched_expression_equations);
                        expression_dependencies.append(&mut matched_expression_dependencies)
                    });

                *dependencies = Dependencies::from(expression_dependencies);

                equations
            }
            StreamExpression::When {
                option,
                present_body,
                present,
                default_body,
                default,
                ref mut dependencies,
                ..
            } => {
                let new_equations = option.normal_form_cascade(identifier_creator);
                let mut option_dependencies = option.get_dependencies().clone();

                let mut present_equations = present.normal_form_cascade(identifier_creator);
                let mut present_dependencies = present.get_dependencies().clone();
                present_body.append(&mut present_equations);
                option_dependencies.append(&mut present_dependencies);

                let mut default_equations = default.normal_form_cascade(identifier_creator);
                let mut default_dependencies = default.get_dependencies().clone();
                default_body.append(&mut default_equations);
                option_dependencies.append(&mut default_dependencies);

                *dependencies = Dependencies::from(option_dependencies);

                new_equations
            }
            _ => vec![],
        }
    }

    fn normal_form_cascade(&mut self, identifier_creator: &mut IdentifierCreator) -> Vec<Equation> {
        let mut new_equations = self.normal_form_root(identifier_creator);
        match self {
            StreamExpression::UnitaryNodeApplication { id: None, .. } => {
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

                new_equations.push(unitary_node_application_equation);

                new_equations
            }
            _ => new_equations,
        }
    }

    fn normal_form_to_signal_call(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
    ) -> Vec<Equation> {
        match self {
            StreamExpression::SignalCall { .. } => vec![],
            _ => {
                let mut equations = self.normal_form_cascade(identifier_creator);

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
mod normal_form_to_signal_call {
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
        let equations = expression.normal_form_to_signal_call(&mut identifier_creator);

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
        let equations = expression.normal_form_to_signal_call(&mut identifier_creator);

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
mod normal_form {
    use std::collections::HashSet;

    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, identifier_creator::IdentifierCreator,
        stream_expression::StreamExpression,
    };

    #[test]
    fn should_change_node_applications_to_be_root_expressions() {
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
                    id: None,
                    node: String::from("my_node"),
                    inputs: vec![
                        (
                            format!("x"),
                            StreamExpression::SignalCall {
                                id: String::from("s"),
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
                                    id: String::from("v"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
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
            dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 0)]),
        };
        let equations = expression.normal_form(&mut identifier_creator);

        // x_2: int = my_node(s, x_1).o;
        let control = Equation {
            scope: Scope::Local,
            id: String::from("x_2"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                id: None,
                node: String::from("my_node"),
                inputs: vec![
                    (
                        format!("x"),
                        StreamExpression::SignalCall {
                            id: String::from("s"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                        },
                    ),
                    (
                        format!("y"),
                        StreamExpression::SignalCall {
                            id: String::from("x_1"),
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
            dependencies: Dependencies::from(vec![(String::from("x_2"), 0)]),
        };
        assert_eq!(expression, control)
    }

    #[test]
    fn should_change_inputs_expressions_to_be_signal_calls() {
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
                    id: None,
                    node: String::from("my_node"),
                    inputs: vec![
                        (
                            format!("x"),
                            StreamExpression::SignalCall {
                                id: String::from("s"),
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
                                    id: String::from("v"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
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
            dependencies: Dependencies::from(vec![(String::from("s"), 0), (String::from("v"), 0)]),
        };
        let equations = expression.normal_form(&mut identifier_creator);

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
                    id: None,
                    node: String::from("my_node"),
                    inputs: vec![
                        (
                            format!("x"),
                            StreamExpression::SignalCall {
                                id: String::from("s"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("s"), 0)]),
                            },
                        ),
                        (
                            format!("y"),
                            StreamExpression::SignalCall {
                                id: String::from("x_1"),
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
        ];
        assert_eq!(equations, control)
    }
}
