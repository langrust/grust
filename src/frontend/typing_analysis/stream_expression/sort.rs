use std::collections::HashMap;

use crate::ast::{
    node_description::NodeDescription, stream_expression::StreamExpression, typedef::Typedef,
};
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};

impl StreamExpression {
    /// Add a [Type] to the sort stream expression.
    pub fn typing_sort(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            StreamExpression::Sort {
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

                        // check it is a sorting function: (element_type, element_type) -> int
                        function_type.eq_check(
                            &Type::Abstract(
                                vec![*element_type.clone(), *element_type.clone()],
                                Box::new(Type::Integer),
                            ),
                            location.clone(),
                            errors,
                        )?;

                        *typing = Some(Type::Array(element_type.clone(), *size));
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
mod typing_sort {
    use crate::ast::{expression::Expression, stream_expression::StreamExpression};
    use crate::common::{location::Location, r#type::Type};
    use std::collections::HashMap;

    #[test]
    fn should_type_sort() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Array(Box::new(Type::Integer), 3));
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("diff"),
            Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut expression = StreamExpression::Sort {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Identifier {
                id: String::from("diff"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::Sort {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: Some(Type::Array(Box::new(Type::Integer), 3)),
                location: Location::default(),
            }),
            function_expression: Expression::Identifier {
                id: String::from("diff"),
                typing: Some(Type::Abstract(
                    vec![Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            },
            typing: Some(Type::Array(Box::new(Type::Integer), 3)),
            location: Location::default(),
        };

        expression
            .typing_sort(
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
    fn should_raise_error_when_expression_not_array() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("diff"),
            Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut expression = StreamExpression::Sort {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Identifier {
                id: String::from("diff"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };

        expression
            .typing_sort(
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
            String::from("diff"),
            Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut expression = StreamExpression::Sort {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Identifier {
                id: String::from("diff"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };

        expression
            .typing_sort(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_function_not_compatible_with_sorting() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Array(Box::new(Type::Integer), 3));
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("diff"),
            Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Boolean)),
        );
        let user_types_context = HashMap::new();

        let mut expression = StreamExpression::Sort {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Identifier {
                id: String::from("diff"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };

        expression
            .typing_sort(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}
