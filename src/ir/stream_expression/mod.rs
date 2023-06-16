use std::collections::HashMap;

use crate::common::{
    color::Color, constant::Constant, graph::Graph, location::Location, pattern::Pattern,
    type_system::Type,
};
use crate::error::Error;
use crate::ir::{expression::Expression, node::Node};

mod array;
mod constant;
mod followed_by;
mod map_application;
mod r#match;
mod node_application;
mod signal_call;
mod structure;
mod when;

#[derive(Debug, PartialEq, Clone)]
/// LanGRust stream expression AST.
pub enum StreamExpression {
    /// Constant stream expression.
    Constant {
        /// The constant.
        constant: Constant,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
    },
    /// Signal call stream expression.
    SignalCall {
        /// Signal identifier.
        id: String,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
    },
    /// Initialized buffer stream expression.
    FollowedBy {
        /// The initialization constant.
        constant: Constant,
        /// The buffered expression.
        expression: Box<StreamExpression>,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
    },
    /// Map application stream expression.
    MapApplication {
        /// The expression applied.
        function_expression: Expression,
        /// The inputs to the expression.
        inputs: Vec<StreamExpression>,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
    },
    /// Node application stream expression.
    NodeApplication {
        /// The node applied.
        node: String,
        /// The inputs to the expression.
        inputs: Vec<StreamExpression>,
        /// The signal retrieved.
        signal: String,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
    },
    /// Structure stream expression.
    Structure {
        /// The structure name.
        name: String,
        /// The fields associated with their expressions.
        fields: Vec<(String, StreamExpression)>,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
    },
    /// Array stream expression.
    Array {
        /// The elements inside the array.
        elements: Vec<StreamExpression>,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
    },
    /// Pattern matching stream expression.
    Match {
        /// The stream expression to match.
        expression: Box<StreamExpression>,
        /// The different matching cases.
        arms: Vec<(Pattern, Option<StreamExpression>, StreamExpression)>,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
    },
    /// When present stream expression.
    When {
        /// The identifier of the value when present
        id: String,
        /// The optional stream expression.
        option: Box<StreamExpression>,
        /// The stream expression when present.
        present: Box<StreamExpression>,
        /// The default stream expression.
        default: Box<StreamExpression>,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
    },
}

impl StreamExpression {
    /// Get dependencies of a stream expression.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ir::{
    ///     equation::Equation, expression::Expression, node::Node, stream_expression::StreamExpression,
    /// };
    /// use grustine::common::{
    ///     constant::Constant, location::Location, scope::Scope, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    ///
    /// let node = Node {
    ///     id: String::from("my_node"),
    ///     is_component: false,
    ///     inputs: vec![
    ///         (String::from("x"), Type::Integer),
    ///         (String::from("y"), Type::Integer),
    ///     ],
    ///     unscheduled_equations: HashMap::from([
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::FollowedBy {
    ///                     constant: Constant::Integer(0),
    ///                     expression: Box::new(StreamExpression::SignalCall {
    ///                         id: String::from("z"),
    ///                         typing: Type::Integer,
    ///                         location: Location::default(),
    ///                     }),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             },
    ///         ),
    ///         (
    ///             String::from("z"),
    ///             Equation {
    ///                 scope: Scope::Local,
    ///                 id: String::from("z"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::FollowedBy {
    ///                     constant: Constant::Integer(1),
    ///                     expression: Box::new(StreamExpression::MapApplication {
    ///                         function_expression: Expression::Call {
    ///                             id: String::from("+"),
    ///                             typing: Type::Abstract(Box::new(Type::Integer), Box::new(Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer)))),
    ///                             location: Location::default(),
    ///                         },
    ///                         inputs: vec![
    ///                             StreamExpression::SignalCall {
    ///                                 id: String::from("x"),
    ///                                 typing: Type::Integer,
    ///                                 location: Location::default(),
    ///                             },
    ///                             StreamExpression::SignalCall {
    ///                                 id: String::from("y"),
    ///                                 typing: Type::Integer,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ],
    ///                         typing: Type::Integer,
    ///                         location: Location::default(),
    ///                     }),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             },
    ///         ),
    ///     ]),
    ///     unitary_nodes: HashMap::new(),
    ///     location: Location::default(),
    /// };
    ///
    /// let mut nodes_context = HashMap::new();
    /// nodes_context.insert(String::from("my_node"), node);
    /// let node = nodes_context.get(&String::from("my_node")).unwrap();
    ///
    /// let graph = node.create_initialized_graph();
    /// let mut nodes_graphs = HashMap::from([(node.id.clone(), graph)]);
    ///
    /// let reduced_graph = node.create_initialized_graph();
    /// let mut nodes_reduced_graphs = HashMap::from([(node.id.clone(), reduced_graph)]);
    ///
    /// let stream_expression = StreamExpression::NodeApplication {
    ///     node: String::from("my_node"),
    ///     inputs: vec![
    ///         StreamExpression::MapApplication {
    ///             function_expression: Expression::Call {
    ///                 id: String::from("f"),
    ///                 typing: Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer)),
    ///                 location: Location::default(),
    ///             },
    ///             inputs: vec![StreamExpression::SignalCall {
    ///                 id: String::from("x"),
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///             }],
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         },
    ///         StreamExpression::Constant {
    ///             constant: Constant::Integer(1),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         },
    ///     ],
    ///     signal: String::from("o"),
    ///     typing: Type::Integer,
    ///     location: Location::default(),
    /// };
    ///
    /// let dependencies = stream_expression
    ///     .get_dependencies(
    ///         &nodes_context,
    ///         &mut nodes_graphs,
    ///         &mut nodes_reduced_graphs,
    ///         &mut errors,
    ///     )
    ///     .unwrap();
    ///
    /// let control = vec![(String::from("x"), 2)];
    ///
    /// assert_eq!(dependencies, control)
    /// ```
    pub fn get_dependencies(
        &self,
        nodes_context: &HashMap<String, Node>,
        nodes_graphs: &mut HashMap<String, Graph<Color>>,
        nodes_reduced_graphs: &mut HashMap<String, Graph<Color>>,
        errors: &mut Vec<Error>,
    ) -> Result<Vec<(String, usize)>, ()> {
        match self {
            StreamExpression::Constant { .. } => self.get_dependencies_constant(),
            StreamExpression::SignalCall { .. } => self.get_dependencies_signal_call(),
            StreamExpression::FollowedBy { .. } => self.get_dependencies_followed_by(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::MapApplication { .. } => self.get_dependencies_map_application(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::Structure { .. } => self.get_dependencies_structure(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::Array { .. } => self.get_dependencies_array(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::Match { .. } => self.get_dependencies_match(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::When { .. } => self.get_dependencies_when(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
            StreamExpression::NodeApplication { .. } => self.get_dependencies_node_application(
                nodes_context,
                nodes_graphs,
                nodes_reduced_graphs,
                errors,
            ),
        }
    }
}

#[cfg(test)]
mod get_dependencies {
    use crate::common::{
        constant::Constant, location::Location, pattern::Pattern, scope::Scope, type_system::Type,
    };
    use crate::ir::{
        equation::Equation, expression::Expression, node::Node, stream_expression::StreamExpression,
    };
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
            .get_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("x"), 0), (String::from("x"), 0)];

        assert_eq!(dependencies, control)
    }

    #[test]
    fn should_get_no_dependencies_from_constant_expression() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Constant {
            constant: Constant::Integer(1),
            typing: Type::Integer,
            location: Location::default(),
        };

        let dependencies = stream_expression
            .get_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

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
            expression: Box::new(StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: String::from("add_one"),
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
            }),
            typing: Type::Integer,
            location: Location::default(),
        };

        let dependencies = stream_expression
            .get_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("x"), 1)];

        assert_eq!(dependencies, control)
    }

    #[test]
    fn should_get_dependencies_of_map_application_inputs_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::MapApplication {
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
        };

        let dependencies = stream_expression
            .get_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("x"), 0)];

        assert_eq!(dependencies, control)
    }

    #[test]
    fn should_get_dependencies_of_match_elements_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Match {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: Type::Structure(String::from("Point")),
                location: Location::default(),
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
                    StreamExpression::SignalCall {
                        id: String::from("z"),
                        typing: Type::Integer,
                        location: Location::default(),
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
                    StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("add_one"),
                            typing: Type::Abstract(
                                Box::new(Type::Integer),
                                Box::new(Type::Integer),
                            ),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            id: String::from("z"),
                            typing: Type::Integer,
                            location: Location::default(),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ),
            ],
            typing: Type::Integer,
            location: Location::default(),
        };

        let mut dependencies = stream_expression
            .get_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();
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
    fn should_get_dependencies_of_match_elements_without_pattern_dependencies() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::Match {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: Type::Structure(String::from("Point")),
                location: Location::default(),
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
                    StreamExpression::SignalCall {
                        id: String::from("y"),
                        typing: Type::Integer,
                        location: Location::default(),
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
                    StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("add_one"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            id: String::from("y"),
                            typing: Type::Integer,
                            location: Location::default(),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ),
            ],
            typing: Type::Integer,
            location: Location::default(),
        };

        let dependencies = stream_expression
            .get_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("p"), 0)];

        assert_eq!(dependencies, control)
    }

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
            .get_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

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
            id: String::from("x"),
            typing: Type::Integer,
            location: Location::default(),
        };

        let dependencies = stream_expression
            .get_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("x"), 0)];

        assert_eq!(dependencies, control)
    }

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
            .get_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("x"), 0), (String::from("x"), 0)];

        assert_eq!(dependencies, control)
    }

    #[test]
    fn should_get_dependencies_of_when_expressions_with_duplicates() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::When {
            id: String::from("x"),
            option: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Type::Integer,
                location: Location::default(),
            }),
            present: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(0),
                typing: Type::Integer,
                location: Location::default(),
            }),
            default: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(1),
                typing: Type::Integer,
                location: Location::default(),
            }),
            typing: Type::Integer,
            location: Location::default(),
        };

        let dependencies = stream_expression
            .get_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("x"), 0)];

        assert_eq!(dependencies, control)
    }

    #[test]
    fn should_get_dependencies_of_when_expressions_without_local_signal() {
        let nodes_context = HashMap::new();
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();
        let mut errors = vec![];

        let stream_expression = StreamExpression::When {
            id: String::from("x"),
            option: Box::new(StreamExpression::SignalCall {
                id: String::from("y"),
                typing: Type::Integer,
                location: Location::default(),
            }),
            present: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Type::Integer,
                location: Location::default(),
            }),
            default: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(1),
                typing: Type::Integer,
                location: Location::default(),
            }),
            typing: Type::Integer,
            location: Location::default(),
        };

        let dependencies = stream_expression
            .get_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                &mut errors,
            )
            .unwrap();

        let control = vec![(String::from("y"), 0)];

        assert_eq!(dependencies, control)
    }
}
