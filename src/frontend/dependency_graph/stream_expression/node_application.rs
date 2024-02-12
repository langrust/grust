use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::graph::{color::Color, neighbor::Label};
use crate::error::{Error, TerminationError};
use crate::hir::{node::Node, stream_expression::StreamExpression};

impl StreamExpression {
    /// Compute dependencies of a node application.
    pub fn compute_node_application_dependencies(
        &self,
        nodes_context: &HashMap<&String, Node>,
        nodes_processus_manager: &mut HashMap<String, HashMap<&String, Color>>,
        nodes_graphs: &mut HashMap<String, DiGraphMap<String, Label>>,
        nodes_reduced_graphs: &mut HashMap<String, DiGraphMap<String, Label>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // dependencies of node application are reduced dependencies of
            // called signal in called node, mapped to inputs
            StreamExpression::NodeApplication {
                node: node_name,
                inputs,
                signal,
                dependencies,
                ..
            } => {
                // get called node structure
                let node = nodes_context.get(node_name).unwrap();

                // create local reduced graphs (because only complete for the called signal)
                let mut local_nodes_reduced_graphs = nodes_reduced_graphs.clone();
                // let mut local_nodes_processus_manager = nodes_processus_manager.clone(); // TODO: see if this is important

                // add dependencies to inputs in the local graphs
                node.add_signal_inputs_dependencies(
                    signal,
                    nodes_context,
                    nodes_processus_manager,
                    nodes_graphs,
                    &mut local_nodes_reduced_graphs,
                    errors,
                )?;

                // get both "real reduced graph" and "local reduced graph" of called node
                let local_reduced_graph = local_nodes_reduced_graphs.get(node_name).unwrap();
                let reduced_graph = nodes_reduced_graphs.get_mut(node_name).unwrap();

                // store computed dependencies (in "local reduced graph") into "real reduced graph"
                local_reduced_graph
                    .edges(signal)
                    .for_each(|(_, id, label)| {
                        reduced_graph.add_edge(signal, id, label.clone());
                    });

                // function "dependencies to inputs" and "input expressions's dependencies"
                // of node application
                dependencies.set(
                    inputs
                        .iter()
                        .zip(&node.inputs)
                        .map(|(input_expression, (input_id, _))| {
                            input_expression.compute_dependencies(
                                nodes_context,
                                nodes_processus_manager,
                                nodes_graphs,
                                nodes_reduced_graphs,
                                errors,
                            )?;
                            Ok(local_reduced_graph.edge_weight(signal, input_id).map_or(
                                Ok(vec![]),
                                |label| {
                                    match label {
                                        Label::Contract => Ok(vec![]), // TODO: do we loose the CREUSOT dependence with the input?
                                        Label::Weight(weight) => Ok(input_expression
                                            .get_dependencies()
                                            .clone()
                                            .into_iter()
                                            .map(|(id, depth)| (id, depth + weight))
                                            .collect()),
                                    }
                                },
                            )?)
                        })
                        .collect::<Result<Vec<Vec<(String, usize)>>, TerminationError>>()?
                        .into_iter()
                        .flatten()
                        .collect::<Vec<(String, usize)>>(),
                );

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod compute_node_application_dependencies {

    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, node::Node, once_cell::OnceCell,
        signal::Signal, stream_expression::StreamExpression,
    };
    use std::collections::HashMap;

    #[test]
    fn should_compute_dependencies_of_node_application_with_mapped_depth() {
        let mut errors = vec![];

        let node = Node {
            contract: Default::default(),
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
                                signal: Signal {
                                    id: String::from("z"),
                                    scope: Scope::Local,
                                },
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::new(),
                            }),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
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
                            expression: Box::new(StreamExpression::FunctionApplication {
                                function_expression: Expression::Identifier {
                                    id: String::from("+"),
                                    typing: Some(Type::Abstract(
                                        vec![Type::Integer, Type::Integer],
                                        Box::new(Type::Integer),
                                    )),
                                    location: Location::default(),
                                },
                                inputs: vec![
                                    StreamExpression::SignalCall {
                                        signal: Signal {
                                            id: String::from("x"),
                                            scope: Scope::Local,
                                        },
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::new(),
                                    },
                                    StreamExpression::SignalCall {
                                        signal: Signal {
                                            id: String::from("y"),
                                            scope: Scope::Local,
                                        },
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::new(),
                                    },
                                ],
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::new(),
                            }),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::new(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut nodes_context = HashMap::new();
        let my_node = String::from("my_node");
        nodes_context.insert(&my_node, node);
        let node = nodes_context.get(&String::from("my_node")).unwrap();

        let graph = node.create_initialized_graph();
        let mut nodes_graphs = HashMap::from([(node.id.clone(), graph)]);

        let reduced_graph = node.create_initialized_graph();
        let mut nodes_reduced_graphs = HashMap::from([(node.id.clone(), reduced_graph)]);

        let processus_manager = node.create_initialized_processus_manager();
        let mut nodes_processus_manager = HashMap::from([(node.id.clone(), processus_manager)]);

        let stream_expression = StreamExpression::NodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::FunctionApplication {
                    function_expression: Expression::Identifier {
                        id: String::from("f"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x"),
                            scope: Scope::Local,
                        },
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
            signal: String::from("o"),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_node_application_dependencies(
                &nodes_context,
                &mut nodes_processus_manager,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();
        let dependencies = stream_expression.get_dependencies().clone();

        let control = vec![(String::from("x"), 2)];

        assert_eq!(dependencies, control)
    }
}
