use std::collections::HashMap;

use crate::ast::{
    node_description::NodeDescription, stream_expression::StreamExpression, typedef::Typedef,
};
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};

impl StreamExpression {
    /// Add a [Type] to the tuple element access stream expression.
    pub fn typing_tuple_element_access(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            StreamExpression::TupleElementAccess {
                expression,
                element_number,
                typing,
                location,
            } => {
                expression.typing(
                    nodes_context,
                    signals_context,
                    global_context,
                    user_types_context,
                    errors,
                )?;

                match expression.get_type().unwrap() {
                    Type::Tuple(elements_type) => {
                        let option_element_type = elements_type.get(*element_number);
                        if let Some(element_type) = option_element_type {
                            *typing = Some(element_type.clone());
                            Ok(())
                        } else {
                            let error = Error::IndexOutOfBounds {
                                location: location.clone(),
                            };
                            errors.push(error);
                            Err(TerminationError)
                        }
                    }
                    given_type => {
                        let error = Error::ExpectTuple {
                            given_type: given_type.clone(),
                            location: location.clone(),
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_tuple_element_access {
    use crate::ast::stream_expression::StreamExpression;
    use crate::ast::typedef::Typedef;
    use crate::common::{location::Location, r#type::Type};
    use std::collections::HashMap;

    #[test]
    fn should_type_tuple_element_access() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(
            String::from("p123"),
            Type::Tuple(vec![
                Type::Structure("Point".to_string()),
                Type::Structure("Point".to_string()),
                Type::Structure("Point".to_string()),
            ]),
        );
        let global_context = HashMap::new();
        let user_types_context = HashMap::from([(
            "Point".to_string(),
            Typedef::Structure {
                id: "Point".to_string(),
                fields: vec![
                    ("x".to_string(), Type::Integer),
                    ("y".to_string(), Type::Integer),
                ],
                location: Location::default(),
            },
        )]);

        let mut expression = StreamExpression::TupleElementAccess {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p123"),
                typing: None,
                location: Location::default(),
            }),
            element_number: 2,
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::TupleElementAccess {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p123"),
                typing: Some(Type::Tuple(vec![
                    Type::Structure("Point".to_string()),
                    Type::Structure("Point".to_string()),
                    Type::Structure("Point".to_string()),
                ])),
                location: Location::default(),
            }),
            element_number: 2,
            typing: Some(Type::Structure("Point".to_string())),
            location: Location::default(),
        };

        expression
            .typing_tuple_element_access(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_when_expression_not_tuple() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("p123"), Type::Structure("Point".to_string()));
        let global_context = HashMap::new();
        let user_types_context = HashMap::from([(
            "Point".to_string(),
            Typedef::Structure {
                id: "Point".to_string(),
                fields: vec![
                    ("x".to_string(), Type::Integer),
                    ("y".to_string(), Type::Integer),
                ],
                location: Location::default(),
            },
        )]);

        let mut expression = StreamExpression::TupleElementAccess {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p123"),
                typing: None,
                location: Location::default(),
            }),
            element_number: 2,
            typing: None,
            location: Location::default(),
        };

        expression
            .typing_tuple_element_access(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_for_index_out_of_bounds() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(
            String::from("p123"),
            Type::Tuple(vec![
                Type::Structure("Point".to_string()),
                Type::Structure("Point".to_string()),
                Type::Structure("Point".to_string()),
            ]),
        );
        let global_context = HashMap::new();
        let user_types_context = HashMap::from([(
            "Point".to_string(),
            Typedef::Structure {
                id: "Point".to_string(),
                fields: vec![
                    ("x".to_string(), Type::Integer),
                    ("y".to_string(), Type::Integer),
                ],
                location: Location::default(),
            },
        )]);

        let mut expression = StreamExpression::TupleElementAccess {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p123"),
                typing: None,
                location: Location::default(),
            }),
            element_number: 3,
            typing: None,
            location: Location::default(),
        };

        expression
            .typing_tuple_element_access(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}
