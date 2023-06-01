use std::collections::HashMap;

use crate::ast::{
    node_description::NodeDescription, stream_expression::StreamExpression, type_system::Type,
    user_defined_type::UserDefinedType,
};
use crate::common::context::Context;
use crate::error::Error;

impl StreamExpression {
    /// Add a [Type] to the structure stream expression.
    pub fn typing_structure(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), Error> {
        match self {
            // the type of the structure is the corresponding structure type
            // if fields match their expected types
            StreamExpression::Structure {
                name,
                fields,
                typing,
                location,
            } => {
                // get the supposed structure type as the user defined it
                let user_type = user_types_context.get_user_type_or_error(
                    name.clone(),
                    location.clone(),
                    errors,
                )?;

                match user_type {
                    UserDefinedType::Structure { .. } => {
                        // type each field
                        fields
                            .into_iter()
                            .map(|(_, stream_expression)| {
                                stream_expression.typing(
                                    nodes_context,
                                    signals_context,
                                    global_context,
                                    user_types_context,
                                    errors,
                                )
                            })
                            .collect::<Vec<Result<(), Error>>>()
                            .into_iter()
                            .collect::<Result<(), Error>>()?;

                        // check that the structure is well defined
                        let well_defined_field =
                            |stream_expression: &StreamExpression,
                             field_type: &Type,
                             errors: &mut Vec<Error>| {
                                let expression_type = stream_expression.get_type().unwrap();
                                expression_type.eq_check(field_type, location.clone(), errors)
                            };
                        user_type.well_defined_structure(fields, well_defined_field, errors)?;

                        *typing = Some(Type::Structure(name.clone()));
                        Ok(())
                    }
                    _ => {
                        let error = Error::ExpectStructure {
                            given_type: user_type.into_type(),
                            location: location.clone(),
                        };
                        errors.push(error.clone());
                        Err(error)
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_structure {
    use crate::ast::{
        constant::Constant, location::Location, stream_expression::StreamExpression,
        type_system::Type, user_defined_type::UserDefinedType,
    };
    use std::collections::HashMap;

    #[test]
    fn should_type_structure_stream_expression() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let signals_context = HashMap::new();
        let global_context = HashMap::new();
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

        let mut stream_expression = StreamExpression::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(2),
                        typing: None,
                        location: Location::default(),
                    },
                ),
            ],
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(2),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
            ],
            typing: Some(Type::Structure(String::from("Point"))),
            location: Location::default(),
        };

        stream_expression
            .typing_structure(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_raise_error_for_additionnal_field_in_structure() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let signals_context = HashMap::new();
        let global_context = HashMap::new();
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

        let mut stream_expression = StreamExpression::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(2),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("z"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(0),
                        typing: None,
                        location: Location::default(),
                    },
                ),
            ],
            typing: None,
            location: Location::default(),
        };

        let error = stream_expression
            .typing_structure(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }

    #[test]
    fn should_raise_error_for_missing_field_in_structure() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let signals_context = HashMap::new();
        let global_context = HashMap::new();
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

        let mut stream_expression = StreamExpression::Structure {
            name: String::from("Point"),
            fields: vec![(
                String::from("x"),
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: None,
                    location: Location::default(),
                },
            )],
            typing: None,
            location: Location::default(),
        };

        let error = stream_expression
            .typing_structure(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }

    #[test]
    fn should_raise_error_for_incompatible_structure() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let signals_context = HashMap::new();
        let global_context = HashMap::new();
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

        let mut stream_expression = StreamExpression::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    StreamExpression::Constant {
                        constant: Constant::Float(2.0),
                        typing: None,
                        location: Location::default(),
                    },
                ),
            ],
            typing: None,
            location: Location::default(),
        };

        let error = stream_expression
            .typing_structure(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }

    #[test]
    fn should_raise_error_when_expect_structure() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let signals_context = HashMap::new();
        let global_context = HashMap::new();
        let mut user_types_context = HashMap::new();
        user_types_context.insert(
            String::from("Color"),
            UserDefinedType::Enumeration {
                id: String::from("Color"),
                elements: vec![
                    String::from("Yellow"),
                    String::from("Blue"),
                    String::from("Green"),
                    String::from("Red"),
                ],
                location: Location::default(),
            },
        );

        let mut stream_expression = StreamExpression::Structure {
            name: String::from("Color"),
            fields: vec![
                (
                    String::from("r"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("g"),
                    StreamExpression::Constant {
                        constant: Constant::Float(2.0),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("b"),
                    StreamExpression::Constant {
                        constant: Constant::Float(2.0),
                        typing: None,
                        location: Location::default(),
                    },
                ),
            ],
            typing: None,
            location: Location::default(),
        };

        let error = stream_expression
            .typing_structure(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }
}
