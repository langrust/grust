use std::collections::HashMap;

use crate::common::graph::{color::Color, Graph};
use crate::error::Error;
use crate::hir::{node::Node, stream_expression::StreamExpression};

impl StreamExpression {
    /// Compute dependencies of a followed by stream expression.
    pub fn compute_followed_by_dependencies(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        match self {
            // dependencies of followed by are dependencies of its following
            // expression, incremented by 1 in depth (because it is a buffer)
            StreamExpression::FollowedBy {
                expression,
                dependencies,
                ..
            } => {
                // propagate dependencies computation
                expression.compute_dependencies(
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;

                // set dependencies with the memory delay
                dependencies.set(
                    expression
                        .get_dependencies()
                        .clone()
                        .into_iter()
                        .map(|(id, depth)| (id, depth + 1))
                        .collect(),
                );

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod compute_followed_by_dependencies {
    use crate::ast::expression::Expression;
    use crate::common::scope::Scope;
    use crate::common::{constant::Constant, location::Location, r#type::Type};
    use crate::hir::{
        dependencies::Dependencies, signal::Signal, stream_expression::StreamExpression,
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
                    typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                    location: Location::default(),
                },
                inputs: vec![StreamExpression::SignalCall {
                    id: String::from("x"),
                    scope: Scope::Local,
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::new(),
                }],
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_followed_by_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();
        let dependencies = stream_expression.get_dependencies().clone();

        let control = vec![(String::from("x"), 1)];

        assert_eq!(dependencies, control)
    }
}
