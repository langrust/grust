use std::collections::HashMap;

use crate::hir::stream_expression::StreamExpression;

impl StreamExpression {
    /// Change node application expressions into unitary node application.
    ///
    /// It removes unused inputs from unitary node application.
    ///
    /// # Example
    ///
    /// Let be a node `my_node` as follows:
    ///
    /// ```GR
    /// node my_node(x: int, y: int) {
    ///     out o1: int = x+y;
    ///     out o2: int = 2*y;
    /// }
    /// ```
    ///
    /// The application of the node `my_node(g-1, v).o2` is changed
    /// to the application of the unitary node `my_node(v).o2`
    pub fn change_node_application_into_unitary_node_application(
        &mut self,
        unitary_nodes_used_inputs: &HashMap<String, HashMap<String, Vec<(String, bool)>>>,
    ) {
        match self {
            StreamExpression::FollowedBy { expression, .. } => expression
                .change_node_application_into_unitary_node_application(unitary_nodes_used_inputs),
            StreamExpression::MapApplication { inputs, .. } => {
                inputs.iter_mut().for_each(|expression| {
                    expression.change_node_application_into_unitary_node_application(
                        unitary_nodes_used_inputs,
                    )
                })
            }
            StreamExpression::NodeApplication {
                node,
                signal,
                inputs,
                typing,
                location,
                dependencies,
            } => {
                let used_inputs = unitary_nodes_used_inputs
                    .get(node)
                    .unwrap()
                    .get(signal)
                    .unwrap();

                let inputs = inputs
                    .into_iter()
                    .zip(used_inputs)
                    .filter_map(|(expression, (input_id, used))| {
                        if *used {
                            Some((input_id.clone(), expression.clone()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                *self = StreamExpression::UnitaryNodeApplication {
                    id: None,
                    node: node.clone(),
                    signal: signal.clone(),
                    inputs,
                    typing: typing.clone(),
                    location: location.clone(),
                    dependencies: dependencies.clone(),
                };
            }
            StreamExpression::UnitaryNodeApplication { id: None, .. } => unreachable!(),
            StreamExpression::Structure { fields, .. } => {
                fields.iter_mut().for_each(|(_, expression)| {
                    expression.change_node_application_into_unitary_node_application(
                        unitary_nodes_used_inputs,
                    )
                })
            }
            StreamExpression::Array { elements, .. } => {
                elements.iter_mut().for_each(|expression| {
                    expression.change_node_application_into_unitary_node_application(
                        unitary_nodes_used_inputs,
                    )
                })
            }
            StreamExpression::Match {
                expression, arms, ..
            } => {
                arms.iter_mut().for_each(|(_, bound, body, expression)| {
                    assert!(body.is_empty());
                    bound.as_mut().map(|expression| {
                        expression.change_node_application_into_unitary_node_application(
                            unitary_nodes_used_inputs,
                        )
                    });
                    expression.change_node_application_into_unitary_node_application(
                        unitary_nodes_used_inputs,
                    )
                });
                expression.change_node_application_into_unitary_node_application(
                    unitary_nodes_used_inputs,
                )
            }
            StreamExpression::When {
                option,
                present_body,
                present,
                default_body,
                default,
                ..
            } => {
                assert!(present_body.is_empty() && default_body.is_empty());
                option.change_node_application_into_unitary_node_application(
                    unitary_nodes_used_inputs,
                );
                present.change_node_application_into_unitary_node_application(
                    unitary_nodes_used_inputs,
                );
                default.change_node_application_into_unitary_node_application(
                    unitary_nodes_used_inputs,
                )
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod change_node_application_into_unitary_node_application {
    use crate::ast::expression::Expression;
    use crate::common::{location::Location, r#type::Type};
    use crate::hir::{dependencies::Dependencies, stream_expression::StreamExpression};
    use std::collections::HashMap;

    #[test]
    fn should_change_node_application_to_unitary_node_application() {
        // my_node(x: int, y: int) { out o1: int = x+y; out o2: int = 2*y; }
        let unitary_nodes_used_inputs = HashMap::from([(
            String::from("my_node"),
            HashMap::from([
                (
                    String::from("o1"),
                    vec![(format!("x"), true), (format!("y"), true)],
                ),
                (
                    String::from("o2"),
                    vec![(format!("x"), false), (format!("y"), true)],
                ),
            ]),
        )]);

        // expression = my_node(g-1, v).o1
        let mut expression = StreamExpression::NodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("-1"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("g"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("g"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("g"), 0)]),
                },
                StreamExpression::SignalCall {
                    id: String::from("v"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                },
            ],
            signal: String::from("o1"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("g"), 0), (String::from("v"), 0)]),
        };
        expression
            .change_node_application_into_unitary_node_application(&unitary_nodes_used_inputs);

        // control = my_node(g-1, v).o1
        let control = StreamExpression::UnitaryNodeApplication {
            id: None,
            node: String::from("my_node"),
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
                            id: String::from("g"),
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
                        id: String::from("v"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                    },
                ),
            ],
            signal: String::from("o1"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("g"), 0), (String::from("v"), 0)]),
        };
        assert_eq!(expression, control);
    }

    #[test]
    fn should_remove_unused_inputs_from_to_unitary_node_application() {
        // my_node(x: int, y: int) { out o1: int = x+y; out o2: int = 2*y; }
        let unitary_nodes_used_inputs = HashMap::from([(
            String::from("my_node"),
            HashMap::from([
                (
                    String::from("o1"),
                    vec![(format!("x"), true), (format!("y"), true)],
                ),
                (
                    String::from("o2"),
                    vec![(format!("x"), false), (format!("y"), true)],
                ),
            ]),
        )]);

        // expression = my_node(g-1, v).o2
        let mut expression = StreamExpression::NodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("-1"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("g"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("g"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("g"), 0)]),
                },
                StreamExpression::SignalCall {
                    id: String::from("v"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                },
            ],
            signal: String::from("o2"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
        };
        expression
            .change_node_application_into_unitary_node_application(&unitary_nodes_used_inputs);

        // control = my_node(v).o2
        let control = StreamExpression::UnitaryNodeApplication {
            id: None,
            node: String::from("my_node"),
            inputs: vec![(
                format!("y"),
                StreamExpression::SignalCall {
                    id: String::from("v"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                },
            )],
            signal: String::from("o2"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
        };
        assert_eq!(expression, control);
    }

    #[test]
    fn should_add_input_identifiers_in_unitary_node_application_inputs() {
        // my_node(x: int, y: int) { out o1: int = x+y; out o2: int = 2*y; }
        let unitary_nodes_used_inputs = HashMap::from([(
            String::from("my_node"),
            HashMap::from([
                (
                    String::from("o1"),
                    vec![(format!("x"), true), (format!("y"), true)],
                ),
                (
                    String::from("o2"),
                    vec![(format!("x"), false), (format!("y"), true)],
                ),
            ]),
        )]);

        // expression = my_node(g-1, v).o2
        let mut expression = StreamExpression::NodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("-1"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("g"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("g"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("g"), 0)]),
                },
                StreamExpression::SignalCall {
                    id: String::from("v"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                },
            ],
            signal: String::from("o2"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
        };
        expression
            .change_node_application_into_unitary_node_application(&unitary_nodes_used_inputs);

        // control = my_node(v).o2
        let control = StreamExpression::UnitaryNodeApplication {
            id: None,
            node: String::from("my_node"),
            inputs: vec![(
                format!("y"),
                StreamExpression::SignalCall {
                    id: String::from("v"),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                },
            )],
            signal: String::from("o2"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
        };
        assert_eq!(expression, control);
    }
}
