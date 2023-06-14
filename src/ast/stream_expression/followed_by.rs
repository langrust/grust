use std::collections::HashMap;

use crate::ast::{
    node::Node, node_description::NodeDescription, stream_expression::StreamExpression,
    type_system::Type, user_defined_type::UserDefinedType,
};
use crate::common::{color::Color, graph::Graph};
use crate::error::Error;

impl StreamExpression {
    /// Add a [Type] to the followed by stream expression.
    pub fn typing_followed_by(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        match self {
            // typing a followed by stream expression consist of getting the type
            // of the constant, typing the next expression and checking types are equal
            StreamExpression::FollowedBy {
                constant,
                expression,
                typing,
                location,
            } => {
                let constant_type = constant.get_type();

                expression.typing(
                    nodes_context,
                    signals_context,
                    global_context,
                    user_types_context,
                    errors,
                )?;

                let expression_type = expression.get_type().unwrap();

                expression_type.eq_check(&constant_type, location.clone(), errors)?;

                *typing = Some(constant_type);
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    /// Get dependencies of a followed by stream expression.
    pub fn get_dependencies_followed_by(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(String, usize)>, ()> {
        match self {
            // dependencies of followed by are dependencies of its following
            // expression, incremented by 1 in depth (because it is a buffer)
            StreamExpression::FollowedBy { expression, .. } => Ok(expression
                .get_dependencies(nodes_context, nodes_graphs, nodes_reduced_graphs, errors)?
                .into_iter()
                .map(|(id, depth)| (id, depth + 1))
                .collect()),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_constant {
    use std::collections::HashMap;

    use crate::ast::{
        constant::Constant, expression::Expression, location::Location,
        stream_expression::StreamExpression, type_system::Type,
    };

    #[test]
    fn should_type_followed_by_stream_expression() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("add_one"),
            Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::FollowedBy {
            constant: Constant::Integer(0),
            expression: Box::new(StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: String::from("add_one"),
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
            }),
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::FollowedBy {
            constant: Constant::Integer(0),
            expression: Box::new(StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: String::from("add_one"),
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
            }),
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        stream_expression
            .typing_followed_by(
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
    fn should_raise_error_for_incompatible_type_in_followed_by() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("add_one"),
            Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::FollowedBy {
            constant: Constant::Float(0.0),
            expression: Box::new(StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: String::from("add_one"),
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
            }),
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing_followed_by(
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
mod get_dependencies_followed_by {
    use crate::ast::{
        constant::Constant, expression::Expression, location::Location,
        stream_expression::StreamExpression,
    };
    use std::collections::HashMap;

    #[test]
    fn should_increment_dependencies_depth_in_followed_by() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::FollowedBy {
            constant: Constant::Float(0.0),
            expression: Box::new(StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: String::from("add_one"),
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
            }),
            typing: None,
            location: Location::default(),
        };

        let dependencies = stream_expression
            .get_dependencies_followed_by(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("x"), 1)];

        assert_eq!(dependencies, control)
    }
}
