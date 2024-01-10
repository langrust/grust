use std::collections::HashMap;

use crate::common::graph::{color::Color, Graph};
use crate::error::{Error, TerminationError};
use crate::hir::{node::Node, stream_expression::StreamExpression};

impl StreamExpression {
    /// Compute dependencies of a tuple element access stream expression.
    pub fn compute_tuple_element_access_dependencies(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // dependencies of tuple element access are dependencies of the accessed expression
            StreamExpression::TupleElementAccess {
                expression,
                dependencies,
                ..
            } => {
                // get accessed expression dependencies
                expression.compute_dependencies(
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                )?;
                let expression_dependencies = expression.get_dependencies().clone();

                // push in tuple element access dependencies
                dependencies.set(expression_dependencies);

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod compute_tuple_element_access_dependencies {
    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, signal::Signal, stream_expression::StreamExpression,
    };
    use std::collections::HashMap;

    #[test]
    fn should_compute_dependencies_of_tuple_element_access() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::TupleElementAccess {
            expression: Box::new(StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("p123"),
                    scope: Scope::Local,
                },
                typing: Type::Tuple(vec![
                    Type::Structure(String::from("Point")),
                    Type::Structure(String::from("Point")),
                    Type::Structure(String::from("Point")),
                ]),
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            element_number: 0,
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_tuple_element_access_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();
        let mut dependencies = stream_expression.get_dependencies().clone();
        dependencies.sort_unstable();

        let control = vec![(String::from("p123"), 0)];

        assert_eq!(dependencies, control)
    }
}
