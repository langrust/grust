use std::collections::HashMap;

use crate::common::{color::Color, graph::Graph};
use crate::error::Error;
use crate::ir::{node::Node, stream_expression::StreamExpression};

impl StreamExpression {
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
mod get_dependencies_followed_by {
    use crate::common::{constant::Constant, location::Location, type_system::Type};
    use crate::ir::{expression::Expression, stream_expression::StreamExpression};
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
                    typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                    location: Location::default(),
                },
                inputs: vec![StreamExpression::SignalCall {
                    id: String::from("x"),
                    typing: Type::Integer,
                    location: Location::default(),
                }],
                typing: Type::Integer,
                location: Location::default(),
            }),
            typing: Type::Integer,
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
