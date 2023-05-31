use std::collections::HashMap;

use crate::ast::{
    stream_expression::{node_description::NodeDescription, StreamExpression},
    type_system::Type,
    user_defined_type::UserDefinedType,
};
use crate::common::context::Context;
use crate::error::Error;

impl StreamExpression {
    /// Add a [Type] to the node application stream expression.
    pub fn typing_node_application(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        elements_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), Error> {
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
                    inputs: node_inputs,
                    outputs: node_outputs,
                    locals: _,
                } = nodes_context.get_node_or_error(node.clone(), location.clone(), errors)?;

                // check inputs and node_inputs have the same length
                if inputs.len() != node_inputs.len() {
                    let error = Error::IncompatibleInputsNumber {
                        given_inputs_number: inputs.len(),
                        expected_inputs_number: node_inputs.len(),
                        location: location.clone(),
                    };
                    errors.push(error.clone());
                    return Err(error);
                }

                // type all inputs and check their types
                inputs
                    .into_iter()
                    .zip(node_inputs)
                    .map(|(input, (_, expected_type))| {
                        input.typing(
                            nodes_context,
                            signals_context,
                            elements_context,
                            user_types_context,
                            errors,
                        )?;
                        let input_type = input.get_type().unwrap();
                        input_type.eq_check(expected_type, location.clone(), errors)
                    })
                    .collect::<Vec<Result<(), Error>>>()
                    .into_iter()
                    .collect::<Result<(), Error>>()?;

                // get the called signal type
                let node_application_type =
                    node_outputs.get_signal_or_error(signal.clone(), location.clone(), errors)?;

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
        constant::Constant, expression::Expression, location::Location,
        stream_expression::node_description::NodeDescription, stream_expression::StreamExpression,
        type_system::Type,
    };
    use std::collections::HashMap;

    #[test]
    fn should_type_node_application_stream_expression() {
        let mut errors = vec![];
        let mut nodes_context = HashMap::new();
        nodes_context.insert(
            String::from("my_node"),
            NodeDescription {
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
        let mut elements_context = HashMap::new();
        elements_context.insert(
            String::from("f"),
            Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::NodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::MapApplication {
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
                StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("f"),
                        typing: Some(Type::Abstract(
                            Box::new(Type::Integer),
                            Box::new(Type::Integer),
                        )),
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
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_raise_error_for_incompatible_node_application() {
        let mut errors = vec![];
        let mut nodes_context = HashMap::new();
        nodes_context.insert(
            String::from("my_node"),
            NodeDescription {
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
        let mut elements_context = HashMap::new();
        elements_context.insert(
            String::from("f"),
            Type::Abstract(Box::new(Type::Integer), Box::new(Type::Float)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::NodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::MapApplication {
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

        let error = stream_expression
            .typing_node_application(
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
