use std::collections::HashMap;

use crate::common::graph::{color::Color, Graph};
use crate::error::Error;
use crate::hir::{node::Node, stream_expression::StreamExpression};

impl StreamExpression {
    /// Compute dependencies of a map application stream expression.
    pub fn compute_dependencies_map_application(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        match self {
            // dependencies of map application are dependencies of its inputs
            StreamExpression::MapApplication {
                inputs,
                dependencies,
                ..
            } => {
                // propagate dependencies computation
                inputs
                    .iter()
                    .map(|input_expression| {
                        input_expression.compute_dependencies(
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
                    inputs
                        .iter()
                        .map(|input_expression| input_expression.get_dependencies().clone())
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
mod compute_dependencies_map_application {
    use crate::common::{location::Location, r#type::Type};
    use crate::hir::dependencies::Dependencies;
    use crate::hir::{expression::Expression, stream_expression::StreamExpression};
    use std::collections::HashMap;

    #[test]
    fn should_compute_dependencies_of_map_application_inputs_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::MapApplication {
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
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
        };

        stream_expression
            .compute_dependencies_map_application(
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
}
