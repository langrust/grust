use std::collections::HashMap;

use crate::common::graph::{color::Color, Graph};
use crate::error::{Error, TerminationError};
use crate::hir::{node::Node, stream_expression::StreamExpression};

impl StreamExpression {
    /// Compute dependencies of a when stream expression.
    pub fn compute_when_dependencies(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // dependencies of when are dependencies of the optional expression
            // plus present and default expressions (without the new local signal)
            StreamExpression::When {
                id: local_signal,
                option,
                present,
                default,
                dependencies,
                ..
            } => {
                // get dependencies of optional expression
                option.compute_dependencies(
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut option_dependencies = option.get_dependencies().clone();

                // get dependencies of present expression without local signal
                present.compute_dependencies(
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut present_dependencies = present
                    .get_dependencies()
                    .clone()
                    .into_iter()
                    .filter(|(signal, _)| !signal.eq(local_signal))
                    .collect();

                // get dependencies of default expression without local signal
                default.compute_dependencies(
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let mut default_dependencies = default
                    .get_dependencies()
                    .clone()
                    .into_iter()
                    .filter(|(signal, _)| !signal.eq(local_signal))
                    .collect();

                // push all dependencies in optional dependencies
                option_dependencies.append(&mut present_dependencies);
                option_dependencies.append(&mut default_dependencies);
                dependencies.set(option_dependencies);

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod compute_when_dependencies {
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::dependencies::Dependencies;
    use crate::hir::{signal::Signal, stream_expression::StreamExpression};
    use std::collections::HashMap;

    #[test]
    fn should_compute_dependencies_of_when_expressions_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::When {
            id: String::from("x"),
            option: Box::new(StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("x"),
                    scope: Scope::Local,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            present_body: vec![],
            present: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(0),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            default_body: vec![],
            default: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(1),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_when_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();
        let dependencies = stream_expression.get_dependencies().clone();

        let control = vec![(String::from("x"), 0)];

        assert_eq!(dependencies, control)
    }

    #[test]
    fn should_compute_dependencies_of_when_expressions_without_local_signal() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::When {
            id: String::from("x"),
            option: Box::new(StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("y"),
                    scope: Scope::Local,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            present_body: vec![],
            present: Box::new(StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("x"),
                    scope: Scope::Local,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            default_body: vec![],
            default: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(1),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_when_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();
        let dependencies = stream_expression.get_dependencies().clone();

        let control = vec![(String::from("y"), 0)];

        assert_eq!(dependencies, control)
    }
}
