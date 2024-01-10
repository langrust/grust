use std::collections::HashMap;

use crate::common::graph::{color::Color, Graph};
use crate::error::{Error, TerminationError};
use crate::hir::{node::Node, stream_expression::StreamExpression};

impl StreamExpression {
    /// Compute dependencies of a zip stream expression.
    pub fn compute_zip_dependencies(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // dependencies of zip are dependencies of its arrays
            StreamExpression::Zip {
                arrays,
                dependencies,
                ..
            } => {
                // propagate dependencies computation
                arrays
                    .iter()
                    .map(|array_expression| {
                        array_expression.compute_dependencies(
                            nodes_context,
                            nodes_graphs,
                            nodes_reduced_graphs,
                            errors,
                        )
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<_, _>>()?;

                // set dependencies
                dependencies.set(
                    arrays
                        .iter()
                        .flat_map(|array_expression| array_expression.get_dependencies().clone())
                        .collect(),
                );

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod compute_zip_dependencies {
    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, signal::Signal, stream_expression::StreamExpression,
    };
    use std::collections::HashMap;

    #[test]
    fn should_compute_dependencies_of_zip_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Zip {
            arrays: vec![
                StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("x"),
                        scope: Scope::Local,
                    },
                    typing: Type::Array(Box::new(Type::Integer), 3),
                    location: Location::default(),
                    dependencies: Dependencies::new(),
                },
                StreamExpression::FunctionApplication {
                    function_expression: Expression::Call {
                        id: String::from("f"),
                        typing: Some(Type::Abstract(
                            vec![Type::Array(Box::new(Type::Integer), 3)],
                            Box::new(Type::Array(Box::new(Type::Float), 3)),
                        )),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x"),
                            scope: Scope::Local,
                        },
                        typing: Type::Array(Box::new(Type::Integer), 3),
                        location: Location::default(),
                        dependencies: Dependencies::new(),
                    }],
                    typing: Type::Array(Box::new(Type::Float), 3),
                    location: Location::default(),
                    dependencies: Dependencies::new(),
                },
                StreamExpression::Array {
                    elements: vec![
                        StreamExpression::Constant {
                            constant: Constant::Integer(1),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        },
                        StreamExpression::Constant {
                            constant: Constant::Integer(1),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        },
                        StreamExpression::Constant {
                            constant: Constant::Integer(1),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        },
                    ],
                    typing: Type::Array(Box::new(Type::Integer), 3),
                    location: Location::default(),
                    dependencies: Dependencies::new(),
                },
            ],
            typing: Type::Array(
                Box::new(Type::Tuple(vec![Type::Integer, Type::Float, Type::Integer])),
                3,
            ),
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_zip_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();
        let dependencies = stream_expression.get_dependencies().clone();

        let control = vec![(String::from("x"), 0), (String::from("x"), 0)];

        assert_eq!(dependencies, control)
    }
}
