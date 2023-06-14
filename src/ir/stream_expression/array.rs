use std::collections::HashMap;

use crate::common::{color::Color, graph::Graph};
use crate::error::Error;
use crate::ir::{node::Node, stream_expression::StreamExpression};

impl StreamExpression {
    /// Get dependencies of an array stream expression.
    pub fn get_dependencies_array(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(String, usize)>, ()> {
        match self {
            // dependencies of array are dependencies of its elements
            StreamExpression::Array { elements, .. } => Ok(elements
                .iter()
                .map(|element_expression| {
                    element_expression.get_dependencies(
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
mod get_dependencies_array {
    use crate::common::{constant::Constant, location::Location, type_system::Type};
    use crate::ir::{expression::Expression, stream_expression::StreamExpression};
    use std::collections::HashMap;

    #[test]
    fn should_get_dependencies_of_array_elements_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Array {
            elements: vec![
                StreamExpression::SignalCall {
                    id: String::from("x"),
                    typing: Type::Integer,
                    location: Location::default(),
                },
                StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("f"),
                        typing: Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer)),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                },
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: Type::Integer,
                    location: Location::default(),
                },
            ],
            typing: Type::Array(Box::new(Type::Integer), 3),
            location: Location::default(),
        };

        let dependencies = stream_expression
            .get_dependencies_array(
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
