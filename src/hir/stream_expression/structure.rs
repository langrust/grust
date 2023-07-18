use std::collections::HashMap;

use crate::common::graph::{color::Color, Graph};
use crate::error::Error;
use crate::hir::{node::Node, stream_expression::StreamExpression};

impl StreamExpression {
    /// Get dependencies of a structure stream expression.
    pub fn get_structure_dependencies(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(String, usize)>, ()> {
        match self {
            // dependencies of structure are dependencies of its fields
            StreamExpression::Structure { fields, .. } => Ok(fields
                .iter()
                .map(|(_, field_expression)| {
                    field_expression.get_dependencies(
                        nodes_context,
                        nodes_graphs,
                        nodes_reduced_graphs,
                        errors,
                    )
                })
                .collect::<Result<Vec<Vec<(String, usize)>>, ()>>()?
                .into_iter()
                .flatten()
                .collect()),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod get_dependencies_structure {
    use crate::common::{location::Location, r#type::Type};
    use crate::hir::stream_expression::StreamExpression;
    use std::collections::HashMap;

    #[test]
    fn should_get_dependencies_of_structure_elements_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ),
            ],
            typing: Type::Integer,
            location: Location::default(),
        };

        let dependencies = stream_expression
            .get_structure_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("x"), 0), (String::from("x"), 0)];

        assert_eq!(dependencies, control)
    }
}
