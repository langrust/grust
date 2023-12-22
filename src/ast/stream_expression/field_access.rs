use std::collections::HashMap;

use crate::ast::{
    node_description::NodeDescription, stream_expression::StreamExpression, typedef::Typedef,
};
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};

impl StreamExpression {
    /// Add a [Type] to the field access stream expression.
    pub fn typing_field_access(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            StreamExpression::FieldAccess {
                expression,
                field,
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
                    Type::Structure(type_id) => match user_types_context.get(type_id).unwrap() {
                        Typedef::Structure { fields, .. } => {
                            let option_field_type = fields
                                .iter()
                                .filter(|(f, _)| f == field)
                                .map(|(_, t)| t.clone())
                                .next();
                            if let Some(field_type) = option_field_type {
                                *typing = Some(field_type);
                                Ok(())
                            } else {
                                let error = Error::UnknownField {
                                    structure_name: type_id.clone(),
                                    field_name: field.clone(),
                                    location: location.clone(),
                                };
                                errors.push(error);
                                Err(TerminationError)
                            }
                        }
                        user_type => {
                            let error = Error::ExpectStructure {
                                given_type: user_type.into_type(),
                                location: location.clone(),
                            };
                            errors.push(error);
                            Err(TerminationError)
                        }
                    },
                    given_type => {
                        let error = Error::ExpectStructure {
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
mod typing_field_access {
    use crate::ast::stream_expression::StreamExpression;
    use crate::ast::typedef::Typedef;
    use crate::common::{location::Location, r#type::Type};
    use std::collections::HashMap;

    #[test]
    fn should_type_field_access() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("p"), Type::Structure("Point".to_string()));
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

        let mut expression = StreamExpression::FieldAccess {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: None,
                location: Location::default(),
            }),
            field: "x".to_string(),
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::FieldAccess {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: Some(Type::Structure("Point".to_string())),
                location: Location::default(),
            }),
            field: "x".to_string(),
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        expression
            .typing_field_access(
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
    fn should_raise_error_when_expression_not_structure() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("p"), Type::Integer);
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

        let mut expression = StreamExpression::FieldAccess {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: None,
                location: Location::default(),
            }),
            field: "x".to_string(),
            typing: None,
            location: Location::default(),
        };

        expression
            .typing_field_access(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_expression_is_enumeration() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("p"), Type::Structure("Point".to_string()));
        let global_context = HashMap::new();
        let user_types_context = HashMap::from([(
            "Point".to_string(),
            Typedef::Enumeration {
                id: "Point".to_string(),
                elements: vec!["A".to_string(), "B".to_string()],
                location: Location::default(),
            },
        )]);

        let mut expression = StreamExpression::FieldAccess {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: None,
                location: Location::default(),
            }),
            field: "x".to_string(),
            typing: None,
            location: Location::default(),
        };

        expression
            .typing_field_access(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_for_unknown_field() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("p"), Type::Structure("Point".to_string()));
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

        let mut expression = StreamExpression::FieldAccess {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: None,
                location: Location::default(),
            }),
            field: "z".to_string(),
            typing: None,
            location: Location::default(),
        };

        expression
            .typing_field_access(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}
