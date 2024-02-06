use std::collections::HashMap;

use crate::common::graph::{color::Color, Graph};
use crate::error::{Error, TerminationError};
use crate::hir::{node::Node, stream_expression::StreamExpression};

mod array;
mod constant;
mod field_access;
mod fold;
mod followed_by;
mod function_application;
mod map;
mod r#match;
mod node_application;
mod signal_call;
mod sort;
mod structure;
mod tuple_element_access;
mod when;
mod zip;

impl StreamExpression {
    /// Compute dependencies of a stream expression.
    ///
    /// # Example
    ///
    /// Considering the following node:
    ///
    /// ```GR
    /// node my_node(x: int, y: int) {
    ///     out o: int = 0 fby z;
    ///     z: int = 1 fby (x + y);
    /// }
    /// ```
    ///
    /// The stream expression `my_node(f(x), 1).o` depends on the signal `x` with
    /// a dependency depth of 2. Indeed, the expression depends on the memory
    /// of the memory of `x` (the signal is behind 2 fby operations).
    pub fn compute_dependencies(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            StreamExpression::Constant { .. } => self.compute_constant_dependencies(),
            StreamExpression::SignalCall { .. } => self.compute_signal_call_dependencies(),
            StreamExpression::FollowedBy { .. } => self.compute_followed_by_dependencies(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::FunctionApplication { .. } => self
                .compute_function_application_dependencies(
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                ),
            StreamExpression::Structure { .. } => self.compute_structure_dependencies(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::Array { .. } => self.compute_array_dependencies(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::Match { .. } => self.compute_match_dependencies(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::When { .. } => self.compute_when_dependencies(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::NodeApplication { .. } => self.compute_node_application_dependencies(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::FieldAccess { .. } => self.compute_field_access_dependencies(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::TupleElementAccess { .. } => self
                .compute_tuple_element_access_dependencies(
                    nodes_context,
                    nodes_graphs,
                    nodes_reduced_graphs,
                    errors,
                ),
            StreamExpression::Map { .. } => self.compute_map_dependencies(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::Fold { .. } => self.compute_fold_dependencies(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::Sort { .. } => self.compute_sort_dependencies(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::Zip { .. } => self.compute_zip_dependencies(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::UnitaryNodeApplication { .. } => unreachable!(),
        }
    }
}

#[cfg(test)]
mod compute_dependencies {

    use crate::ast::expression::Expression;
    use crate::common::{
        constant::Constant, location::Location, pattern::Pattern, r#type::Type, scope::Scope,
    };
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, node::Node, once_cell::OnceCell,
        signal::Signal, stream_expression::StreamExpression,
    };
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
                    signal: Signal {
                        id: String::from("x"),
                        scope: Scope::Local,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::new(),
                },
                StreamExpression::FunctionApplication {
                    function_expression: Expression::Call {
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
            typing: Type::Array(Box::new(Type::Integer), 3),
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_dependencies(
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

    #[test]
    fn should_compute_no_dependencies_from_constant_expression() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Constant {
            constant: Constant::Integer(1),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();
        let dependencies = stream_expression.get_dependencies().clone();

        let control = vec![];

        assert_eq!(dependencies, control)
    }

    #[test]
    fn should_increment_dependencies_depth_in_followed_by() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::FollowedBy {
            constant: Constant::Float(0.0),
            expression: Box::new(StreamExpression::FunctionApplication {
                function_expression: Expression::Call {
                    id: String::from("add_one"),
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
            }),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_dependencies(
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

    #[test]
    fn should_compute_dependencies_of_function_application_inputs_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::FunctionApplication {
            function_expression: Expression::Call {
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
        };

        stream_expression
            .compute_dependencies(
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

    #[test]
    fn should_compute_dependencies_of_match_elements_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Match {
            expression: Box::new(StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("p"),
                    scope: Scope::Local,
                },
                typing: Type::Structure(String::from("Point")),
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            arms: vec![
                (
                    Pattern::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                Pattern::Constant {
                                    constant: Constant::Integer(0),
                                    location: Location::default(),
                                },
                            ),
                            (
                                String::from("y"),
                                Pattern::Default {
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                    None,
                    vec![],
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("z"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::new(),
                    },
                ),
                (
                    Pattern::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                Pattern::Default {
                                    location: Location::default(),
                                },
                            ),
                            (
                                String::from("y"),
                                Pattern::Default {
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                    None,
                    vec![],
                    StreamExpression::FunctionApplication {
                        function_expression: Expression::Call {
                            id: String::from("add_one"),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("z"),
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
                ),
            ],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();
        let mut dependencies = stream_expression.get_dependencies().clone();
        dependencies.sort_unstable();

        let mut control = vec![
            (String::from("p"), 0),
            (String::from("z"), 0),
            (String::from("z"), 0),
        ];
        control.sort_unstable();

        assert_eq!(dependencies, control)
    }

    #[test]
    fn should_compute_dependencies_of_match_elements_without_pattern_dependencies() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Match {
            expression: Box::new(StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("p"),
                    scope: Scope::Local,
                },
                typing: Type::Structure(String::from("Point")),
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            arms: vec![
                (
                    Pattern::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                Pattern::Constant {
                                    constant: Constant::Integer(0),
                                    location: Location::default(),
                                },
                            ),
                            (
                                String::from("y"),
                                Pattern::Identifier {
                                    name: String::from("y"),
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                    None,
                    vec![],
                    StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("y"),
                            scope: Scope::Local,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::new(),
                    },
                ),
                (
                    Pattern::Structure {
                        name: String::from("Point"),
                        fields: vec![
                            (
                                String::from("x"),
                                Pattern::Default {
                                    location: Location::default(),
                                },
                            ),
                            (
                                String::from("y"),
                                Pattern::Identifier {
                                    name: String::from("y"),
                                    location: Location::default(),
                                },
                            ),
                        ],
                        location: Location::default(),
                    },
                    None,
                    vec![],
                    StreamExpression::FunctionApplication {
                        function_expression: Expression::Call {
                            id: String::from("add_one"),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("y"),
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
                ),
            ],
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();
        let dependencies = stream_expression.get_dependencies().clone();

        let control = vec![(String::from("p"), 0)];

        assert_eq!(dependencies, control)
    }

    #[test]
    fn should_compute_dependencies_of_node_application_with_mapped_depth() {
        let mut errors = vec![];

        let node = Node { contracts: (vec![], vec![]),
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
                                function_expression: Expression::Call {
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
        nodes_context.insert(String::from("my_node"), node);
        let node = nodes_context.get(&String::from("my_node")).unwrap();

        let graph = node.create_initialized_graph();
        let mut nodes_graphs = HashMap::from([(node.id.clone(), graph)]);

        let reduced_graph = node.create_initialized_graph();
        let mut nodes_reduced_graphs = HashMap::from([(node.id.clone(), reduced_graph)]);

        let stream_expression = StreamExpression::NodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::FunctionApplication {
                    function_expression: Expression::Call {
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
            .compute_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();
        let dependencies = stream_expression.get_dependencies().clone();

        let control = vec![(String::from("x"), 2)];

        assert_eq!(dependencies, control)
    }

    #[test]
    fn should_dependencies_of_signal_call_is_signal_with_zero_depth() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::SignalCall {
            signal: Signal {
                id: String::from("x"),
                scope: Scope::Local,
            },
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_dependencies(
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

    #[test]
    fn should_compute_dependencies_of_structure_elements_with_duplicates() {
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
            .compute_dependencies(
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

    #[test]
    fn should_compute_dependencies_of_when_expressions_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::When {
            id: String::from("x"),
            option: Box::new(StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("x"),
                    scope: Scope::Local,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            present_body: vec![],
            present: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(0),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            default_body: vec![],
            default: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(1),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_dependencies(
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

    #[test]
    fn should_compute_dependencies_of_when_expressions_without_local_signal() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::When {
            id: String::from("x"),
            option: Box::new(StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("y"),
                    scope: Scope::Local,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            present_body: vec![],
            present: Box::new(StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("x"),
                    scope: Scope::Local,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            default_body: vec![],
            default: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(1),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();
        let dependencies = stream_expression.get_dependencies().clone();

        let control = vec![(String::from("y"), 0)];

        assert_eq!(dependencies, control)
    }

    #[test]
    fn should_compute_dependencies_of_field_access() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::FieldAccess {
            expression: Box::new(StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("p"),
                    scope: Scope::Local,
                },
                typing: Type::Structure(String::from("Point")),
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            field: "x".to_string(),
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();
        let mut dependencies = stream_expression.get_dependencies().clone();
        dependencies.sort_unstable();

        let control = vec![(String::from("p"), 0)];

        assert_eq!(dependencies, control)
    }

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
            .compute_dependencies(
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

    #[test]
    fn should_compute_dependencies_of_map() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Map {
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
                id: String::from("f"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Float))),
                location: Location::default(),
            },
            typing: Type::Array(Box::new(Type::Float), 3),
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_dependencies(
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

    #[test]
    fn should_compute_dependencies_of_fold() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Fold {
            expression: Box::new(StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("a"),
                    scope: Scope::Local,
                },
                typing: Type::Array(Box::new(Type::Integer), 3),
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            initialization_expression: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(0),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::new(),
            }),
            function_expression: Expression::Call {
                id: String::from("sum"),
                typing: Some(Type::Abstract(
                    vec![Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            },
            typing: Type::Integer,
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_dependencies(
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
            .compute_dependencies(
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

    #[test]
    fn should_compute_dependencies_of_zip_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Zip {
            arrays: vec![
                StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("x"),
                        scope: Scope::Local,
                    },
                    typing: Type::Array(Box::new(Type::Integer), 3),
                    location: Location::default(),
                    dependencies: Dependencies::new(),
                },
                StreamExpression::FunctionApplication {
                    function_expression: Expression::Call {
                        id: String::from("f"),
                        typing: Some(Type::Abstract(
                            vec![Type::Array(Box::new(Type::Integer), 3)],
                            Box::new(Type::Array(Box::new(Type::Float), 3)),
                        )),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x"),
                            scope: Scope::Local,
                        },
                        typing: Type::Array(Box::new(Type::Integer), 3),
                        location: Location::default(),
                        dependencies: Dependencies::new(),
                    }],
                    typing: Type::Array(Box::new(Type::Float), 3),
                    location: Location::default(),
                    dependencies: Dependencies::new(),
                },
                StreamExpression::Array {
                    elements: vec![
                        StreamExpression::Constant {
                            constant: Constant::Integer(1),
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
                },
            ],
            typing: Type::Array(
                Box::new(Type::Tuple(vec![Type::Integer, Type::Float, Type::Integer])),
                3,
            ),
            location: Location::default(),
            dependencies: Dependencies::new(),
        };

        stream_expression
            .compute_dependencies(
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
