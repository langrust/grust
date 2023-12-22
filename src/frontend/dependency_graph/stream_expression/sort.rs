use std::collections::HashMap;

use crate::common::graph::{color::Color, Graph};
use crate::error::{Error, TerminationError};
use crate::hir::{node::Node, stream_expression::StreamExpression};

impl StreamExpression {
    /// Compute dependencies of a sort stream expression.
    pub fn compute_sort_dependencies(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // dependencies of sort are dependencies of the sorted expression
            StreamExpression::Sort {
                expression,
                dependencies,
                ..
            } => {
                // get sorted expression dependencies
                expression.compute_dependencies(
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let expression_dependencies = expression.get_dependencies().clone();

                // push in sort dependencies
                dependencies.set(expression_dependencies);

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod compute_sort_dependencies {
    use crate::ast::expression::Expression;
    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, signal::Signal, stream_expression::StreamExpression,
    };
    use std::collections::HashMap;

    #[test]
    fn should_compute_dependencies_of_sort() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Sort {
            expression: Box::new(StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("a"),
                    scope: Scope::Local,
                },
                typing: Type::Array(Box::new(Type::Integer), 3),
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            function_expression: Expression::Call {
                id: String::from("diff"),
                typing: Some(Type::Abstract(
                    vec![Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            },
            typing: Type::Array(Box::new(Type::Float), 3),
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_sort_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();
        let mut dependencies = stream_expression.get_dependencies().clone();
        dependencies.sort_unstable();

        let control = vec![(String::from("a"), 0)];

        assert_eq!(dependencies, control)
    }
}
