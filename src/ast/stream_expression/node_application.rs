use std::collections::HashMap;

use crate::ast::{
    node::Node, node_description::NodeDescription, stream_expression::StreamExpression,
    type_system::Type, user_defined_type::UserDefinedType,
};
use crate::common::graph::neighbor::Neighbor;
use crate::common::{color::Color, context::Context, graph::Graph};
use crate::error::Error;

impl StreamExpression {
    /// Add a [Type] to the node application stream expression.
    pub fn typing_node_application(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
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
                    return Err(());
                }

                // check inputs and node_inputs have the same length
                if inputs.len() != node_inputs.len() {
                    let error = Error::IncompatibleInputsNumber {
                        given_inputs_number: inputs.len(),
                        expected_inputs_number: node_inputs.len(),
                        location: location.clone(),
                    };
                    errors.push(error);
                    return Err(());
                }

                // type all inputs and check their types
                inputs
                    .into_iter()
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
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // get the called signal type
                let node_application_type =
                    node_outputs.get_signal_or_error(signal, location.clone(), errors)?;

                *typing = Some(node_application_type.clone());
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    /// Get dependencies of a node application.
    pub fn get_dependencies_node_application(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(String, usize)>, ()> {
        match self {
            // dependencies of node application are reduced dependencies of
            // called signal in called node, mapped to inputs
            StreamExpression::NodeApplication {
                node: node_name,
                inputs,
                signal,
                location,
                ..
            } => {
                // get called node structure
                let node = nodes_context.get_node_or_error(node_name, location.clone(), errors)?;

                // create local reduced graphs (because only complete for the called signal)
                let mut local_nodes_reduced_graphs = nodes_reduced_graphs.clone();

                // add dependencies to inputs in the local graphs
                node.add_signal_inputs_dependencies(
                    signal,
                    nodes_context,
                    nodes_graphs,
                    &mut local_nodes_reduced_graphs,
                    errors,
                )?;

                // get both "real reduced graph" and "local reduced graph" of called node
                let local_reduced_graph = local_nodes_reduced_graphs.get(node_name).unwrap();
                let reduced_graph = nodes_reduced_graphs.get_mut(node_name).unwrap();

                // store computed dependencies (in "local reduced graph") into "real reduced graph"
                local_reduced_graph
                    .get_vertex(signal)
                    .get_neighbors()
                    .into_iter()
                    .for_each(|Neighbor { id, weight }| reduced_graph.add_edge(signal, id, weight));

                // map "dependencies to inputs" and "input expressions's dependencies"
                // of node application
                Ok(inputs
                    .iter()
                    .zip(&node.inputs)
                    .map(|(input_expression, (input_id, _))| {
                        Ok(local_reduced_graph
                            .get_weights(signal, input_id)
                            .iter()
                            .map(|weight| {
                                Ok(input_expression
                                    .get_dependencies(
                                        nodes_context,
                                        nodes_graphs,
                                        nodes_reduced_graphs,
                                        errors,
                                    )?
                                    .into_iter()
                                    .map(|(id, depth)| (id, depth + weight))
                                    .collect())
                            })
                            .collect::<Result<Vec<Vec<(String, usize)>>, ()>>()?
                            .into_iter()
                            .flatten()
                            .collect::<Vec<(String, usize)>>())
                    })
                    .collect::<Result<Vec<Vec<(String, usize)>>, ()>>()?
                    .into_iter()
                    .flatten()
                    .collect::<Vec<(String, usize)>>())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_node_application {
    use crate::ast::{
        constant::Constant, expression::Expression, location::Location,
        node_description::NodeDescription, stream_expression::StreamExpression, type_system::Type,
    };
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
            Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::NodeApplication {
            node: String::from("my_component"),
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

#[cfg(test)]
mod get_dependencies_node_application {
    use crate::ast::{
        constant::Constant, equation::Equation, expression::Expression, location::Location,
        node::Node, scope::Scope, stream_expression::StreamExpression, type_system::Type,
    };
    use std::collections::HashMap;

    #[test]
    fn should_get_dependencies_of_node_application_with_mapped_depth() {
        let mut errors = vec![];

        let node = Node {
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            equations: vec![
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::FollowedBy {
                            constant: Constant::Integer(0),
                            expression: Box::new(StreamExpression::SignalCall {
                                id: String::from("z"),
                                typing: None,
                                location: Location::default(),
                            }),
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("z"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("z"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::FollowedBy {
                            constant: Constant::Integer(1),
                            expression: Box::new(StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: String::from("+"),
                                    typing: None,
                                    location: Location::default(),
                                },
                                inputs: vec![
                                    StreamExpression::SignalCall {
                                        id: String::from("x"),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                    StreamExpression::SignalCall {
                                        id: String::from("y"),
                                        typing: None,
                                        location: Location::default(),
                                    },
                                ],
                                typing: None,
                                location: Location::default(),
                            }),
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        };

        let mut nodes_context = HashMap::new();
        nodes_context.insert(String::from("my_node"), node);
        let node = nodes_context.get(&String::from("my_node")).unwrap();

        let graph = node.create_initialized_graph(&mut errors).unwrap();
        let mut nodes_graphs = HashMap::from([(node.id.clone(), graph)]);

        let reduced_graph = node.create_initialized_graph(&mut errors).unwrap();
        let mut nodes_reduced_graphs = HashMap::from([(node.id.clone(), reduced_graph)]);

        let stream_expression = StreamExpression::NodeApplication {
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

        let dependencies = stream_expression
            .get_dependencies_node_application(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("x"), 2)];

        assert_eq!(dependencies, control)
    }
}
