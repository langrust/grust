use std::collections::HashMap;

use crate::common::graph::{color::Color, Graph};
use crate::error::Error;
use crate::ir::{node::Node, stream_expression::StreamExpression};

impl StreamExpression {
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
                // get dependencies of optional expression
                let mut option_dependencies = option.get_dependencies(
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;

                // get dependencies of present expression without local signal
                let mut present_dependencies = present
                    .get_dependencies(nodes_context, nodes_graphs, nodes_reduced_graphs, errors)?
                    .into_iter()
                    .filter(|(signal, _)| !signal.eq(local_signal))
                    .collect();

                // get dependencies of default expression without local signal
                let mut default_dependencies = default
                    .get_dependencies(nodes_context, nodes_graphs, nodes_reduced_graphs, errors)?
                    .into_iter()
                    .filter(|(signal, _)| !signal.eq(local_signal))
                    .collect();

                // push all dependencies in optional dependencies
                option_dependencies.append(&mut present_dependencies);
                option_dependencies.append(&mut default_dependencies);

                // return optional dependencies
                Ok(option_dependencies)
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod get_dependencies_when {
    use crate::common::{constant::Constant, location::Location, type_system::Type};
    use crate::ir::stream_expression::StreamExpression;
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
                typing: Type::Integer,
                location: Location::default(),
            }),
            present_body: vec![],
            present: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(0),
                typing: Type::Integer,
                location: Location::default(),
            }),
            default_body: vec![],
            default: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(1),
                typing: Type::Integer,
                location: Location::default(),
            }),
            typing: Type::Integer,
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
                typing: Type::Integer,
                location: Location::default(),
            }),
            present_body: vec![],
            present: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Type::Integer,
                location: Location::default(),
            }),
            default_body: vec![],
            default: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(1),
                typing: Type::Integer,
                location: Location::default(),
            }),
            typing: Type::Integer,
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
