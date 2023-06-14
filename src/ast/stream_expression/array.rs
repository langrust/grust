use std::collections::HashMap;

use crate::ast::{
    node::Node, node_description::NodeDescription, stream_expression::StreamExpression,
};
use crate::common::{
    color::Color, graph::Graph, type_system::Type, user_defined_type::UserDefinedType,
};
use crate::error::Error;

impl StreamExpression {
    /// Add a [Type] to the array stream expression.
    pub fn typing_array(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        match self {
            // an array is composed of `n` elements of the same type `t` and
            // its type is `[t; n]`
            StreamExpression::Array {
                elements,
                typing,
                location,
            } => {
                elements
                    .into_iter()
                    .map(|element| {
                        element.typing(
                            nodes_context,
                            signals_context,
                            global_context,
                            user_types_context,
                            errors,
                        )
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                let first_type = elements[0].get_type().unwrap();
                elements
                    .iter()
                    .map(|element| {
                        let element_type = element.get_type().unwrap();
                        element_type.eq_check(first_type, location.clone(), errors)
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                let array_type = Type::Array(Box::new(first_type.clone()), elements.len());

                *typing = Some(array_type);
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    /// Get dependencies of an array stream expression.
    pub fn get_dependencies_array(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(String, usize)>, ()> {
        match self {
            // dependencies of array are dependencies of its elements
            StreamExpression::Array { elements, .. } => Ok(elements
                .iter()
                .map(|element_expression| {
                    element_expression.get_dependencies(
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
mod typing_array {
    use crate::ast::{expression::Expression, stream_expression::StreamExpression};
    use crate::common::{constant::Constant, location::Location, type_system::Type};
    use std::collections::HashMap;

    #[test]
    fn should_type_array_stream_expression() {
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

        let mut stream_expression = StreamExpression::Array {
            elements: vec![
                StreamExpression::SignalCall {
                    id: String::from("x"),
                    typing: None,
                    location: Location::default(),
                },
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
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::Array {
            elements: vec![
                StreamExpression::SignalCall {
                    id: String::from("x"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
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
            typing: Some(Type::Array(Box::new(Type::Integer), 3)),
            location: Location::default(),
        };

        stream_expression
            .typing_array(
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
    fn should_raise_error_for_multiple_types_array() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Array {
            elements: vec![
                StreamExpression::SignalCall {
                    id: String::from("x"),
                    typing: None,
                    location: Location::default(),
                },
                StreamExpression::Constant {
                    constant: Constant::Float(1.0),
                    typing: None,
                    location: Location::default(),
                },
            ],
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing_array(
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
mod get_dependencies_array {
    use crate::ast::{expression::Expression, stream_expression::StreamExpression};
    use crate::common::{constant::Constant, location::Location};
    use std::collections::HashMap;

    #[test]
    fn should_get_dependencies_of_array_elements_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Array {
            elements: vec![
                StreamExpression::SignalCall {
                    id: String::from("x"),
                    typing: None,
                    location: Location::default(),
                },
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
            typing: None,
            location: Location::default(),
        };

        let dependencies = stream_expression
            .get_dependencies_array(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("x"), 0), (String::from("x"), 0)];

        assert_eq!(dependencies, control)
    }
}
