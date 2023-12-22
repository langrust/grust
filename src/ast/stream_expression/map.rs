use std::collections::HashMap;

use crate::ast::{
    node_description::NodeDescription, stream_expression::StreamExpression, typedef::Typedef,
};
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};

impl StreamExpression {
    /// Add a [Type] to the map stream expression.
    pub fn typing_map(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            StreamExpression::Map {
                expression,
                function_expression,
                typing,
                location,
            } => {
                // type the expression
                expression.typing(
                    nodes_context,
                    signals_context,
                    global_context,
                    user_types_context,
                    errors,
                )?;

                // verify it is an array
                match expression.get_type().unwrap() {
                    Type::Array(element_type, size) => {
                        // type the function expression
                        let elements_context = global_context.clone();
                        function_expression.typing(
                            global_context,
                            &elements_context,
                            user_types_context,
                            errors,
                        )?;
                        let function_type = function_expression.get_type_mut().unwrap();

                        // apply the function type to the type of array's elements
                        let new_element_type = function_type.apply(
                            vec![*element_type.clone()],
                            location.clone(),
                            errors,
                        )?;

                        *typing = Some(Type::Array(Box::new(new_element_type), *size));
                        Ok(())
                    }
                    given_type => {
                        let error = Error::ExpectArray {
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
mod typing_map {
    use crate::ast::{expression::Expression, stream_expression::StreamExpression};
    use crate::common::{location::Location, r#type::Type};
    use std::collections::HashMap;

    #[test]
    fn should_type_map() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Array(Box::new(Type::Integer), 3));
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Float)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Map {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::Map {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: Some(Type::Array(Box::new(Type::Integer), 3)),
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Float))),
                location: Location::default(),
            },
            typing: Some(Type::Array(Box::new(Type::Float), 3)),
            location: Location::default(),
        };

        stream_expression
            .typing_map(
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
    fn should_raise_error_when_expression_not_array() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Float)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Map {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing_map(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_function_not_compatible_with_array_elements() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Array(Box::new(Type::Boolean), 3));
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Float)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Map {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing_map(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}
