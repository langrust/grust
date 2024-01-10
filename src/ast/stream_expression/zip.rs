use std::collections::HashMap;

use crate::ast::{
    node_description::NodeDescription, stream_expression::StreamExpression, typedef::Typedef,
};
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};

impl StreamExpression {
    /// Add a [Type] to the zip stream expression.
    pub fn typing_zip(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            StreamExpression::Zip {
                arrays,
                typing,
                location,
            } => {
                if arrays.len() == 0 {
                    let error = Error::ExpectInput {
                        location: location.clone(),
                    };
                    errors.push(error);
                    return Err(TerminationError);
                }

                arrays
                    .iter_mut()
                    .map(|array| {
                        array.typing(
                            nodes_context,
                            signals_context,
                            global_context,
                            user_types_context,
                            errors,
                        )
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                let length = match arrays[0].get_type().unwrap() {
                    Type::Array(_, n) => Ok(n),
                    ty => {
                        let error = Error::ExpectArray {
                            given_type: ty.clone(),
                            location: location.clone(),
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }?;
                let tuple_types = arrays
                    .iter()
                    .map(|array| match array.get_type().unwrap() {
                        Type::Array(ty, n) if n == length => Ok(*ty.clone()),
                        Type::Array(_, n) => {
                            let error = Error::IncompatibleLength {
                                given_length: *n,
                                expected_length: *length,
                                location: location.clone(),
                            };
                            errors.push(error);
                            Err(TerminationError)
                        }
                        ty => {
                            let error = Error::ExpectArray {
                                given_type: ty.clone(),
                                location: location.clone(),
                            };
                            errors.push(error);
                            Err(TerminationError)
                        }
                    })
                    .collect::<Vec<Result<Type, TerminationError>>>()
                    .into_iter()
                    .collect::<Result<Vec<Type>, TerminationError>>()?;

                let array_type = if tuple_types.len() > 1 {
                    Type::Array(Box::new(Type::Tuple(tuple_types)), *length)
                } else {
                    Type::Array(Box::new(tuple_types.get(0).unwrap().clone()), *length)
                };

                *typing = Some(array_type);
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_zip {
    use crate::ast::stream_expression::StreamExpression;
    use crate::common::{location::Location, r#type::Type};
    use std::collections::HashMap;

    #[test]
    fn should_type_zip_with_one_array() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Array(Box::new(Type::Integer), 3));
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Zip {
            arrays: vec![StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }],
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::Zip {
            arrays: vec![StreamExpression::SignalCall {
                id: String::from("a"),
                typing: Some(Type::Array(Box::new(Type::Integer), 3)),
                location: Location::default(),
            }],
            typing: Some(Type::Array(Box::new(Type::Integer), 3)),
            location: Location::default(),
        };

        stream_expression
            .typing_zip(
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
    fn should_type_zip_with_multiple_arrays() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Array(Box::new(Type::Integer), 3));
        signals_context.insert(String::from("b"), Type::Array(Box::new(Type::Float), 3));
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Zip {
            arrays: vec![
                StreamExpression::SignalCall {
                    id: String::from("a"),
                    typing: None,
                    location: Location::default(),
                },
                StreamExpression::SignalCall {
                    id: String::from("b"),
                    typing: None,
                    location: Location::default(),
                },
            ],
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::Zip {
            arrays: vec![
                StreamExpression::SignalCall {
                    id: String::from("a"),
                    typing: Some(Type::Array(Box::new(Type::Integer), 3)),
                    location: Location::default(),
                },
                StreamExpression::SignalCall {
                    id: String::from("b"),
                    typing: Some(Type::Array(Box::new(Type::Float), 3)),
                    location: Location::default(),
                },
            ],
            typing: Some(Type::Array(
                Box::new(Type::Tuple(vec![Type::Integer, Type::Float])),
                3,
            )),
            location: Location::default(),
        };

        stream_expression
            .typing_zip(
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
    fn should_raise_error_when_zero_array() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let signals_context = HashMap::new();
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Zip {
            arrays: vec![],
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing_zip(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_not_array() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Integer);
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Zip {
            arrays: vec![StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }],
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing_zip(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_incompatible_length() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Array(Box::new(Type::Integer), 3));
        signals_context.insert(String::from("b"), Type::Array(Box::new(Type::Float), 5));
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Zip {
            arrays: vec![
                StreamExpression::SignalCall {
                    id: String::from("a"),
                    typing: None,
                    location: Location::default(),
                },
                StreamExpression::SignalCall {
                    id: String::from("b"),
                    typing: None,
                    location: Location::default(),
                },
            ],
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing_zip(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}
