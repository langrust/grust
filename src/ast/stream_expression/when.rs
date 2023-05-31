use std::collections::HashMap;

use crate::ast::{
    stream_expression::{node_description::NodeDescription, StreamExpression}, type_system::Type, user_defined_type::UserDefinedType,
};
use crate::error::Error;

impl StreamExpression {
    /// Add a [Type] to the when stream expression.
    pub fn typing_when(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        elements_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), Error> {
        match self {
            // the type of a when stream expression is the type of both the default and
            // the present stream expressions
            StreamExpression::When {
                id,
                option,
                present,
                default,
                typing,
                location,
            } => {
                option.typing(
                    nodes_context,
                    signals_context,
                    elements_context,
                    user_types_context,
                    errors,
                )?;

                let option_type = option.get_type().unwrap();
                match option_type {
                    Type::Option(unwraped_type) => {
                        let mut local_context = signals_context.clone();
                        local_context.insert(id.clone(), *unwraped_type.clone());

                        present.typing(
                            nodes_context,
                            &local_context,
                            elements_context,
                            user_types_context,
                            errors,
                        )?;
                        default.typing(
                            nodes_context,
                            signals_context,
                            elements_context,
                            user_types_context,
                            errors,
                        )?;

                        let present_type = present.get_type().unwrap();
                        let default_type = default.get_type().unwrap();

                        *typing = Some(present_type.clone());
                        default_type.eq_check(present_type, location.clone(), errors)
                    }
                    _ => {
                        let error = Error::ExpectOption {
                            given_type: option_type.clone(),
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
mod typing_when {
    use crate::ast::{
        constant::Constant, location::Location, stream_expression::StreamExpression,
        type_system::Type,
    };
    use std::collections::HashMap;

    #[test]
    fn should_type_when_expression() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Option(Box::new(Type::Integer)));
        let elements_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::When {
            id: String::from("x"),
            option: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            present: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            default: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(1),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::When {
            id: String::from("x"),
            option: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Some(Type::Option(Box::new(Type::Integer))),
                location: Location::default(),
            }),
            present: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            default: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(1),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        stream_expression
            .typing_when(
                &nodes_context,
                &signals_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_raise_error_for_incompatible_when() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Option(Box::new(Type::Integer)));
        let elements_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::When {
            id: String::from("x"),
            option: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            present: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            default: Box::new(StreamExpression::Constant {
                constant: Constant::Float(1.0),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };

        let error = stream_expression
            .typing_when(
                &nodes_context,
                &signals_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }
}
