use std::collections::HashMap;

use crate::ast::{
    node::Node, node_description::NodeDescription, stream_expression::StreamExpression,
    type_system::Type, user_defined_type::UserDefinedType,
};
use crate::common::{color::Color, graph::Graph};
use crate::error::Error;

impl StreamExpression {
    /// Add a [Type] to the map application stream expression.
    pub fn typing_map_application(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        match self {
            // a map application expression type is the result of the application
            // of the inputs types to the abstraction/function type
            StreamExpression::MapApplication {
                function_expression,
                inputs,
                typing,
                location,
            } => {
                // type the function expression
                let elements_context = global_context.clone();
                let test_typing_function_expression = function_expression.typing(
                    global_context,
                    &elements_context,
                    user_types_context,
                    errors,
                );
                // type all inputs
                let test_typing_inputs = inputs
                    .into_iter()
                    .map(|input| {
                        input.typing(
                            nodes_context,
                            signals_context,
                            global_context,
                            user_types_context,
                            errors,
                        )
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>();

                // test if there were some errors
                test_typing_function_expression?;
                test_typing_inputs?;

                // compute the application type
                let application_type = inputs.iter().fold(
                    Ok(function_expression.get_type().unwrap().clone()),
                    |current_typing, input| {
                        let abstraction_type = current_typing.unwrap().clone();
                        let input_type = input.get_type().unwrap().clone();
                        Ok(abstraction_type.apply(input_type, location.clone(), errors)?)
                    },
                )?;

                *typing = Some(application_type);
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    /// Get dependencies of a map application stream expression.
    pub fn get_dependencies_map_application(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(String, usize)>, ()> {
        match self {
            // dependencies of map application are dependencies of its inputs
            StreamExpression::MapApplication { inputs, .. } => Ok(inputs
                .iter()
                .map(|input_expression| {
                    input_expression.get_dependencies(
                        nodes_context,
                        nodes_graphs,
                        nodes_reduced_graphs,
                        errors,
                    )
                })
                .collect::<Result<Vec<Vec<(String, usize)>>, ()>>()?
                .into_iter()
                .flatten()
                .collect()),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_map_application {
    use crate::ast::{
        expression::Expression, location::Location, stream_expression::StreamExpression,
        type_system::Type,
    };
    use std::collections::HashMap;

    #[test]
    fn should_type_map_application_stream_expression() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::MapApplication {
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
        };
        let control = StreamExpression::MapApplication {
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
        };

        stream_expression
            .typing_map_application(
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
    fn should_raise_error_for_incompatible_map_application() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(Box::new(Type::Float), Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::MapApplication {
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
        };

        stream_expression
            .typing_map_application(
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
mod get_dependencies_map_application {
    use crate::ast::{
        expression::Expression, location::Location, stream_expression::StreamExpression,
    };
    use std::collections::HashMap;

    #[test]
    fn should_get_dependencies_of_map_application_inputs_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::MapApplication {
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
        };

        let dependencies = stream_expression
            .get_dependencies_map_application(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("x"), 0)];

        assert_eq!(dependencies, control)
    }
}
