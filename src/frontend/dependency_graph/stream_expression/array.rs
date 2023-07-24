use std::collections::HashMap;

use crate::common::graph::{color::Color, Graph};
use crate::error::Error;
use crate::hir::{node::Node, stream_expression::StreamExpression};

impl StreamExpression {
    /// Compute dependencies of an array stream expression.
    pub fn compute_dependencies_array(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        match self {
            // dependencies of array are dependencies of its elements
            StreamExpression::Array {
                elements,
                dependencies,
                ..
            } => {
                // propagate dependencies computation
                elements
                    .iter()
                    .map(|element_expression| {
                        element_expression.compute_dependencies(
                            nodes_context,
                            nodes_graphs,
                            nodes_reduced_graphs,
                            errors,
                        )
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<_, _>>()?;

                // set dependencies
                dependencies.set(
                    elements
                        .iter()
                        .map(|element_expression| element_expression.get_dependencies().clone())
                        .flatten()
                        .collect(),
                );

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod compute_dependencies_array {
    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location, r#type::Type};
    use crate::hir::{dependencies::Dependencies, stream_expression::StreamExpression};
    use std::collections::HashMap;

    #[test]
    fn should_compute_dependencies_of_array_elements_with_duplicates() {
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
                    dependencies: Dependencies::new(),
                },
                StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("f"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::new(),
                    }],
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
        };

        stream_expression
            .compute_dependencies_array(
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
