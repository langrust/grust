use std::collections::HashMap;

use crate::common::graph::{color::Color, Graph};
use crate::error::Error;
use crate::ir::{node::Node, stream_expression::StreamExpression};

impl StreamExpression {
    /// Get dependencies of a match stream expression.
    pub fn get_dependencies_match(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(String, usize)>, ()> {
        match self {
            // dependencies of match are dependencies of matched expression and
            // dependencies of arms (without new signals defined in patterns)
            StreamExpression::Match {
                expression, arms, ..
            } => {
                // compute arms dependencies
                let mut arms_dependencies = arms
                    .iter()
                    .map(|(pattern, bound, _, arm_expression)| {
                        // get local signals defined in pattern
                        let local_signals = pattern.local_identifiers();

                        // get arm expression dependencies
                        let mut arm_dependencies = arm_expression
                            .get_dependencies(
                                nodes_context,
                                nodes_graphs,
                                nodes_reduced_graphs,
                                errors,
                            )?
                            .into_iter()
                            .filter(|(signal, _)| !local_signals.contains(signal))
                            .collect::<Vec<(String, usize)>>();

                        // get bound dependencies
                        let mut bound_dependencies = bound
                            .as_ref()
                            .map_or(Ok(vec![]), |bound_expression| {
                                bound_expression.get_dependencies(
                                    nodes_context,
                                    nodes_graphs,
                                    nodes_reduced_graphs,
                                    errors,
                                )
                            })?
                            .into_iter()
                            .filter(|(signal, _)| !local_signals.contains(signal))
                            .collect();

                        // push all dependencies in arm dependencies
                        arm_dependencies.append(&mut bound_dependencies);

                        // return arm dependencies
                        Ok(arm_dependencies)
                    })
                    .collect::<Result<Vec<Vec<(String, usize)>>, ()>>()?
                    .into_iter()
                    .flatten()
                    .collect::<Vec<(String, usize)>>();

                // get matched expression dependencies
                let mut expression_dependencies = expression.get_dependencies(
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;

                // push all dependencies in arms dependencies
                arms_dependencies.append(&mut expression_dependencies);

                // return arms dependencies
                Ok(arms_dependencies)
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod get_dependencies_match {
    use crate::common::{
        constant::Constant, location::Location, pattern::Pattern, type_system::Type,
    };
    use crate::ir::{expression::Expression, stream_expression::StreamExpression};
    use std::collections::HashMap;

    #[test]
    fn should_get_dependencies_of_match_elements_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Match {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: Type::Structure(String::from("Point")),
                location: Location::default(),
            }),
            arms: vec![
                (
                    Pattern::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                Pattern::Constant {
                                    constant: Constant::Integer(0),
                                    location: Location::default(),
                                },
                            ),
                            (
                                String::from("y"),
                                Pattern::Default {
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                    None,
                    vec![],
                    StreamExpression::SignalCall {
                        id: String::from("z"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ),
                (
                    Pattern::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                Pattern::Default {
                                    location: Location::default(),
                                },
                            ),
                            (
                                String::from("y"),
                                Pattern::Default {
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                    None,
                    vec![],
                    StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("add_one"),
                            typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            id: String::from("z"),
                            typing: Type::Integer,
                            location: Location::default(),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ),
            ],
            typing: Type::Integer,
            location: Location::default(),
        };

        let mut dependencies = stream_expression
            .get_dependencies_match(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();
        dependencies.sort_unstable();

        let mut control = vec![
            (String::from("p"), 0),
            (String::from("z"), 0),
            (String::from("z"), 0),
        ];
        control.sort_unstable();

        assert_eq!(dependencies, control)
    }

    #[test]
    fn should_get_dependencies_of_match_elements_without_pattern_dependencies() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Match {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: Type::Structure(String::from("Point")),
                location: Location::default(),
            }),
            arms: vec![
                (
                    Pattern::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                Pattern::Constant {
                                    constant: Constant::Integer(0),
                                    location: Location::default(),
                                },
                            ),
                            (
                                String::from("y"),
                                Pattern::Identifier {
                                    name: String::from("y"),
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                    None,
                    vec![],
                    StreamExpression::SignalCall {
                        id: String::from("y"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ),
                (
                    Pattern::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                Pattern::Default {
                                    location: Location::default(),
                                },
                            ),
                            (
                                String::from("y"),
                                Pattern::Identifier {
                                    name: String::from("y"),
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                    None,
                    vec![],
                    StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("add_one"),
                            typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            id: String::from("y"),
                            typing: Type::Integer,
                            location: Location::default(),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ),
            ],
            typing: Type::Integer,
            location: Location::default(),
        };

        let dependencies = stream_expression
            .get_dependencies_match(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("p"), 0)];

        assert_eq!(dependencies, control)
    }
}
