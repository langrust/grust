use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::graph::color::Color;
use crate::common::graph::neighbor::Label;
use crate::error::{Error, TerminationError};
use crate::hir::{node::Node, stream_expression::StreamExpression};

impl StreamExpression {
    /// Compute dependencies of a structure stream expression.
    pub fn compute_structure_dependencies(
        &self,
        nodes_context: &HashMap<&String, Node>,
        nodes_processus_manager: &mut HashMap<String, HashMap<&String, Color>>,
        nodes_graphs: &mut HashMap<String, DiGraphMap<String, Label>>,
        nodes_reduced_graphs: &mut HashMap<String, DiGraphMap<String, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // dependencies of structure are dependencies of its fields
            StreamExpression::Structure {
                fields,
                dependencies,
                ..
            } => {
                // propagate dependencies computation
                fields
                    .iter()
                    .map(|(_, field_expression)| {
                        field_expression.compute_dependencies(
                            nodes_context,
                            nodes_processus_manager,
                            nodes_graphs,
                            nodes_reduced_graphs,
                            errors,
                        )
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<_, _>>()?;

                // set dependencies
                dependencies.set(
                    fields
                        .iter()
                        .flat_map(|(_, field_expression)| {
                            field_expression.get_dependencies().clone()
                        })
                        .collect(),
                );

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod compute_structure_dependencies {
    use crate::common::{location::Location, r#type::Type, scope::Scope};
    use crate::hir::dependencies::Dependencies;
    use crate::hir::{signal::Signal, stream_expression::StreamExpression};
    use std::collections::HashMap;

    #[test]
    fn should_compute_dependencies_of_structure_elements_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_processus_manager = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::new(),
                    },
                ),
                (
                    String::from("y"),
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::new(),
                    },
                ),
            ],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_structure_dependencies(
                &nodes_context,
                &mut nodes_processus_manager,
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
