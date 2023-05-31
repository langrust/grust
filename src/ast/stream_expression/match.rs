use std::collections::HashMap;

use crate::ast::{
    node_description::NodeDescription, stream_expression::StreamExpression, type_system::Type,
    user_defined_type::UserDefinedType,
};
use crate::error::Error;

impl StreamExpression {
    /// Add a [Type] to the match stream expression.
    pub fn typing_match(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        elements_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), Error> {
        match self {
            // the type of a match stream expression is the type of all branches expressions
            StreamExpression::Match {
                expression,
                arms,
                typing,
                location,
            } => {
                expression.typing(
                    nodes_context,
                    signals_context,
                    elements_context,
                    user_types_context,
                    errors,
                )?;

                let expression_type = expression.get_type().unwrap();
                arms.into_iter()
                    .map(|(pattern, optional_test_expression, arm_expression)| {
                        let mut local_context = signals_context.clone();
                        pattern.construct_context(
                            expression_type,
                            &mut local_context,
                            user_types_context,
                            errors,
                        )?;

                        let optional_test_expression_typing_test = optional_test_expression
                            .as_mut()
                            .map_or(Ok(()), |stream_expression| {
                                stream_expression.typing(
                                    nodes_context,
                                    &local_context,
                                    elements_context,
                                    user_types_context,
                                    errors,
                                )?;
                                stream_expression.get_type().unwrap().eq_check(
                                    &Type::Boolean,
                                    location.clone(),
                                    errors,
                                )
                            });

                        let arm_expression_typing_test = arm_expression.typing(
                            nodes_context,
                            &local_context,
                            elements_context,
                            user_types_context,
                            errors,
                        );

                        optional_test_expression_typing_test?;
                        arm_expression_typing_test
                    })
                    .collect::<Vec<Result<(), Error>>>()
                    .into_iter()
                    .collect::<Result<(), Error>>()?;

                let first_type = arms[0].2.get_type().unwrap();
                arms.iter()
                    .map(|(_, _, arm_expression)| {
                        let arm_expression_type = arm_expression.get_type().unwrap();
                        arm_expression_type.eq_check(first_type, location.clone(), errors)
                    })
                    .collect::<Vec<Result<(), Error>>>()
                    .into_iter()
                    .collect::<Result<(), Error>>()?;

                // todo: patterns should be exhaustive
                *typing = Some(first_type.clone());
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_match {
    use crate::ast::{
        constant::Constant, expression::Expression, location::Location, pattern::Pattern,
        stream_expression::StreamExpression, type_system::Type, user_defined_type::UserDefinedType,
    };
    use std::collections::HashMap;

    #[test]
    fn should_type_match_structure_stream_expression() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("p"), Type::Structure(String::from("Point")));
        let mut elements_context = HashMap::new();
        elements_context.insert(
            String::from("add_one"),
            Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer)),
        );
        let mut user_types_context = HashMap::new();
        user_types_context.insert(
            String::from("Point"),
            UserDefinedType::Structure {
                id: String::from("Point"),
                fields: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer),
                ],
                location: Location::default(),
            },
        );

        let mut stream_expression = StreamExpression::Match {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: None,
                location: Location::default(),
            }),
            arms: vec![
                (
                    Pattern::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                Pattern::Constant {
                                    constant: Constant::Integer(0),
                                    location: Location::default(),
                                },
                            ),
                            (
                                String::from("y"),
                                Pattern::Identifier {
                                    name: String::from("y"),
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                    None,
                    StreamExpression::SignalCall {
                        id: String::from("y"),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    Pattern::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                Pattern::Default {
                                    location: Location::default(),
                                },
                            ),
                            (
                                String::from("y"),
                                Pattern::Identifier {
                                    name: String::from("y"),
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                    None,
                    StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("add_one"),
                            typing: None,
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            id: String::from("y"),
                            typing: None,
                            location: Location::default(),
                        }],
                        typing: None,
                        location: Location::default(),
                    },
                ),
            ],
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::Match {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: Some(Type::Structure(String::from("Point"))),
                location: Location::default(),
            }),
            arms: vec![
                (
                    Pattern::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                Pattern::Constant {
                                    constant: Constant::Integer(0),
                                    location: Location::default(),
                                },
                            ),
                            (
                                String::from("y"),
                                Pattern::Identifier {
                                    name: String::from("y"),
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                    None,
                    StreamExpression::SignalCall {
                        id: String::from("y"),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
                (
                    Pattern::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                Pattern::Default {
                                    location: Location::default(),
                                },
                            ),
                            (
                                String::from("y"),
                                Pattern::Identifier {
                                    name: String::from("y"),
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                    None,
                    StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("add_one"),
                            typing: Some(Type::Abstract(
                                Box::new(Type::Integer),
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            id: String::from("y"),
                            typing: Some(Type::Integer),
                            location: Location::default(),
                        }],
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
            ],
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        stream_expression
            .typing_match(
                &nodes_context,
                &signals_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(stream_expression, control);
    }
}
