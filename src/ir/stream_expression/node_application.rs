use std::collections::HashMap;

use crate::common::{
    color::Color,
    context::Context,
    graph::{neighbor::Neighbor, Graph},
};
use crate::error::Error;
use crate::ir::{node::Node, stream_expression::StreamExpression};

impl StreamExpression {
    /// Get dependencies of a node application.
    pub fn get_dependencies_node_application(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(String, usize)>, ()> {
        match self {
            // dependencies of node application are reduced dependencies of
            // called signal in called node, mapped to inputs
            StreamExpression::NodeApplication {
                node: node_name,
                inputs,
                signal,
                location,
                ..
            } => {
                // get called node structure
                let node = nodes_context.get_node_or_error(node_name, location.clone(), errors)?;

                // create local reduced graphs (because only complete for the called signal)
                let mut local_nodes_reduced_graphs = nodes_reduced_graphs.clone();

                // add dependencies to inputs in the local graphs
                node.add_signal_inputs_dependencies(
                    signal,
                    nodes_context,
                    nodes_graphs,
                    &mut local_nodes_reduced_graphs,
                    errors,
                )?;

                // get both "real reduced graph" and "local reduced graph" of called node
                let local_reduced_graph = local_nodes_reduced_graphs.get(node_name).unwrap();
                let reduced_graph = nodes_reduced_graphs.get_mut(node_name).unwrap();

                // store computed dependencies (in "local reduced graph") into "real reduced graph"
                local_reduced_graph
                    .get_vertex(signal)
                    .get_neighbors()
                    .into_iter()
                    .for_each(|Neighbor { id, weight }| reduced_graph.add_edge(signal, id, weight));

                // map "dependencies to inputs" and "input expressions's dependencies"
                // of node application
                Ok(inputs
                    .iter()
                    .zip(&node.inputs)
                    .map(|(input_expression, (input_id, _))| {
                        Ok(local_reduced_graph
                            .get_weights(signal, input_id)
                            .iter()
                            .map(|weight| {
                                Ok(input_expression
                                    .get_dependencies(
                                        nodes_context,
                                        nodes_graphs,
                                        nodes_reduced_graphs,
                                        errors,
                                    )?
                                    .into_iter()
                                    .map(|(id, depth)| (id, depth + weight))
                                    .collect())
                            })
                            .collect::<Result<Vec<Vec<(String, usize)>>, ()>>()?
                            .into_iter()
                            .flatten()
                            .collect::<Vec<(String, usize)>>())
                    })
                    .collect::<Result<Vec<Vec<(String, usize)>>, ()>>()?
                    .into_iter()
                    .flatten()
                    .collect::<Vec<(String, usize)>>())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod get_dependencies_node_application {
    use crate::common::{constant::Constant, location::Location, scope::Scope, type_system::Type};
    use crate::ir::{
        equation::Equation, expression::Expression, node::Node, stream_expression::StreamExpression,
    };
    use std::collections::HashMap;

    #[test]
    fn should_get_dependencies_of_node_application_with_mapped_depth() {
        let mut errors = vec![];

        let node = Node {
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::FollowedBy {
                            constant: Constant::Integer(0),
                            expression: Box::new(StreamExpression::SignalCall {
                                id: String::from("z"),
                                typing: Type::Integer,
                                location: Location::default(),
                            }),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("z"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("z"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::FollowedBy {
                            constant: Constant::Integer(1),
                            expression: Box::new(StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: String::from("+"),
                                    typing: Type::Abstract(
                                        Box::new(Type::Integer),
                                        Box::new(Type::Abstract(
                                            Box::new(Type::Integer),
                                            Box::new(Type::Integer),
                                        )),
                                    ),
                                    location: Location::default(),
                                },
                                inputs: vec![
                                    StreamExpression::SignalCall {
                                        id: String::from("x"),
                                        typing: Type::Integer,
                                        location: Location::default(),
                                    },
                                    StreamExpression::SignalCall {
                                        id: String::from("y"),
                                        typing: Type::Integer,
                                        location: Location::default(),
                                    },
                                ],
                                typing: Type::Integer,
                                location: Location::default(),
                            }),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
        };

        let mut nodes_context = HashMap::new();
        nodes_context.insert(String::from("my_node"), node);
        let node = nodes_context.get(&String::from("my_node")).unwrap();

        let graph = node.create_initialized_graph();
        let mut nodes_graphs = HashMap::from([(node.id.clone(), graph)]);

        let reduced_graph = node.create_initialized_graph();
        let mut nodes_reduced_graphs = HashMap::from([(node.id.clone(), reduced_graph)]);

        let stream_expression = StreamExpression::NodeApplication {
            node: String::from("my_node"),
            inputs: vec![
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
            signal: String::from("o"),
            typing: Type::Integer,
            location: Location::default(),
        };

        let dependencies = stream_expression
            .get_dependencies_node_application(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("x"), 2)];

        assert_eq!(dependencies, control)
    }
}
