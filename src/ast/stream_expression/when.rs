use std::collections::HashMap;

use crate::ast::{
    node::Node, node_description::NodeDescription, stream_expression::StreamExpression,
    type_system::Type, user_defined_type::UserDefinedType,
};
use crate::common::{color::Color, graph::Graph};
use crate::error::Error;

impl StreamExpression {
    /// Add a [Type] to the when stream expression.
    pub fn typing_when(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
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
                    global_context,
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
                            global_context,
                            user_types_context,
                            errors,
                        )?;
                        default.typing(
                            nodes_context,
                            signals_context,
                            global_context,
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
                        errors.push(error);
                        Err(())
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    /// Get dependencies of a when stream expression.
    pub fn get_dependencies_when(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(String, usize)>, ()> {
        match self {
            // dependencies of when are dependencies of the optional expression
            // plus present and default expressions (without the new local signal)
            StreamExpression::When {
                id: local_signal,
                option,
                present,
                default,
                ..
            } => {
                let mut option_dependencies = option.get_dependencies(
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut present_dependencies = present
                    .get_dependencies(nodes_context, nodes_graphs, nodes_reduced_graphs, errors)?
                    .into_iter()
                    .filter(|(signal, _)| !signal.eq(local_signal))
                    .collect();
                let mut default_dependencies = default
                    .get_dependencies(nodes_context, nodes_graphs, nodes_reduced_graphs, errors)?
                    .into_iter()
                    .filter(|(signal, _)| !signal.eq(local_signal))
                    .collect();
                option_dependencies.append(&mut present_dependencies);
                option_dependencies.append(&mut default_dependencies);
                Ok(option_dependencies)
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
        let global_context = HashMap::new();
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
                &global_context,
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
        let global_context = HashMap::new();
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

        stream_expression
            .typing_when(
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
mod get_dependencies_when {
    use crate::ast::{constant::Constant, location::Location, stream_expression::StreamExpression};
    use std::collections::HashMap;

    #[test]
    fn should_get_dependencies_of_when_expressions_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::When {
            id: String::from("x"),
            option: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            present: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(0),
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

        let dependencies = stream_expression
            .get_dependencies_when(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("x"), 0)];

        assert_eq!(dependencies, control)
    }

    #[test]
    fn should_get_dependencies_of_when_expressions_without_local_signal() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::When {
            id: String::from("x"),
            option: Box::new(StreamExpression::SignalCall {
                id: String::from("y"),
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

        let dependencies = stream_expression
            .get_dependencies_when(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("y"), 0)];

        assert_eq!(dependencies, control)
    }
}
