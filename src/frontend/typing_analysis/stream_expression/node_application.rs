use std::collections::HashMap;

use crate::ast::{
    node_description::NodeDescription, stream_expression::StreamExpression, typedef::Typedef,
};
use crate::common::{context::Context, r#type::Type};
use crate::error::{Error, TerminationError};

impl StreamExpression {
    /// Add a [Type] to the node application stream expression.
    pub fn typing_node_application(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // a node application expression type is the called signal when
            // the inputs types matches the called node inputs types
            StreamExpression::NodeApplication {
                node,
                inputs,
                signal,
                typing,
                location,
            } => {
                // get the called node description
                let NodeDescription {
                    is_component,
                    inputs: node_inputs,
                    outputs: node_outputs,
                    locals: _,
                } = nodes_context.get_node_or_error(node, location.clone(), errors)?;

                // if component raise error: component can not be called
                if *is_component {
                    let error = Error::ComponentCall {
                        name: node.clone(),
                        location: location.clone(),
                    };
                    errors.push(error);
                    return Err(TerminationError);
                }

                // check inputs and node_inputs have the same length
                if inputs.len() != node_inputs.len() {
                    let error = Error::IncompatibleInputsNumber {
                        given_inputs_number: inputs.len(),
                        expected_inputs_number: node_inputs.len(),
                        location: location.clone(),
                    };
                    errors.push(error);
                    return Err(TerminationError);
                }

                // type all inputs and check their types
                inputs
                    .iter_mut()
                    .zip(node_inputs)
                    .map(|(input, (_, expected_type))| {
                        input.typing(
                            nodes_context,
                            signals_context,
                            global_context,
                            user_types_context,
                            errors,
                        )?;
                        let input_type = input.get_type().unwrap();
                        input_type.eq_check(expected_type, location.clone(), errors)
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                // get the called signal type
                let node_application_type =
                    node_outputs.get_signal_or_error(signal, location.clone(), errors)?;

                *typing = Some(node_application_type.clone());
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_node_application {
    use crate::ast::{
        expression::Expression, node_description::NodeDescription,
        stream_expression::StreamExpression,
    };
    use crate::common::{constant::Constant, location::Location, r#type::Type};
    use std::collections::HashMap;

    #[test]
    fn should_type_node_application_stream_expression() {
        let mut errors = vec![];
        let mut nodes_context = HashMap::new();
        nodes_context.insert(
            String::from("my_node"),
            NodeDescription {
                is_component: false,
                inputs: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer),
                ],
                outputs: HashMap::from([(String::from("o"), Type::Integer)]),
                locals: HashMap::new(),
            },
        );
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::NodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::FunctionApplication {
                    function_expression: Expression::Call {
                        id: String::from("f"),
                        typing: None,
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: None,
                        location: Location::default(),
                    }],
                    typing: None,
                    location: Location::default(),
                },
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: None,
                    location: Location::default(),
                },
            ],
            signal: String::from("o"),
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::NodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::FunctionApplication {
                    function_expression: Expression::Call {
                        id: String::from("f"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    }],
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
            ],
            signal: String::from("o"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        stream_expression
            .typing_node_application(
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
    fn should_raise_error_for_component_call() {
        let mut errors = vec![];
        let mut nodes_context = HashMap::new();
        nodes_context.insert(
            String::from("my_component"),
            NodeDescription {
                is_component: true,
                inputs: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer),
                ],
                outputs: HashMap::from([(String::from("o"), Type::Integer)]),
                locals: HashMap::new(),
            },
        );
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::NodeApplication {
            node: String::from("my_component"),
            inputs: vec![
                StreamExpression::FunctionApplication {
                    function_expression: Expression::Call {
                        id: String::from("f"),
                        typing: None,
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: None,
                        location: Location::default(),
                    }],
                    typing: None,
                    location: Location::default(),
                },
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: None,
                    location: Location::default(),
                },
            ],
            signal: String::from("o"),
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing_node_application(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_for_incompatible_node_application() {
        let mut errors = vec![];
        let mut nodes_context = HashMap::new();
        nodes_context.insert(
            String::from("my_node"),
            NodeDescription {
                is_component: false,
                inputs: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer),
                ],
                outputs: HashMap::from([(String::from("o"), Type::Integer)]),
                locals: HashMap::new(),
            },
        );
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Float)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::NodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::FunctionApplication {
                    function_expression: Expression::Call {
                        id: String::from("f"),
                        typing: None,
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: None,
                        location: Location::default(),
                    }],
                    typing: None,
                    location: Location::default(),
                },
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: None,
                    location: Location::default(),
                },
            ],
            signal: String::from("o"),
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing_node_application(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}
