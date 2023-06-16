use std::collections::HashMap;

use crate::common::scope::Scope;
use crate::common::{
    color::Color, constant::Constant, graph::Graph, location::Location, pattern::Pattern,
    type_system::Type,
};
use crate::error::Error;
use crate::ir::{equation::Equation, expression::Expression, node::Node};

use super::identifier_creator::IdentifierCreator;

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
        arms: Vec<(
            Pattern,
            Option<StreamExpression>,
            Vec<Equation>,
            StreamExpression,
        )>,
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
        /// The body of present case when normalized.
        present_body: Vec<Equation>,
        /// The stream expression when present.
        present: Box<StreamExpression>,
        /// The body of default case when normalized.
        default_body: Vec<Equation>,
        /// The default stream expression.
        default: Box<StreamExpression>,
        /// Stream Expression type.
        typing: Type,
        /// Stream expression location.
        location: Location,
    },
}

impl StreamExpression {
    /// Get the reference to the stream expression's typing.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::ir::stream_expression::StreamExpression;
    /// use grustine::common::{constant::Constant, location::Location, type_system::Type};
    ///
    /// let mut stream_expression = StreamExpression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Type::Integer,
    ///     location: Location::default(),
    /// };
    /// let typing = stream_expression.get_type();
    /// assert_eq!(typing, &Type::Integer)
    /// ```
    pub fn get_type(&self) -> &Type {
        match self {
            StreamExpression::Constant { typing, .. }
            | StreamExpression::SignalCall { typing, .. }
            | StreamExpression::FollowedBy { typing, .. }
            | StreamExpression::MapApplication { typing, .. }
            | StreamExpression::NodeApplication { typing, .. }
            | StreamExpression::Structure { typing, .. }
            | StreamExpression::Array { typing, .. }
            | StreamExpression::Match { typing, .. }
            | StreamExpression::When { typing, .. } => typing,
        }
    }

    /// Get the reference to the stream expression's location.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::ir::stream_expression::StreamExpression;
    /// use grustine::common::{constant::Constant, location::Location, type_system::Type};
    ///
    /// let mut stream_expression = StreamExpression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Type::Integer,
    ///     location: Location::default(),
    /// };
    /// let location = stream_expression.get_location();
    /// assert_eq!(location, &Location::default())
    /// ```
    pub fn get_location(&self) -> &Location {
        match self {
            StreamExpression::Constant { location, .. }
            | StreamExpression::SignalCall { location, .. }
            | StreamExpression::FollowedBy { location, .. }
            | StreamExpression::MapApplication { location, .. }
            | StreamExpression::NodeApplication { location, .. }
            | StreamExpression::Structure { location, .. }
            | StreamExpression::Array { location, .. }
            | StreamExpression::Match { location, .. }
            | StreamExpression::When { location, .. } => location,
        }
    }

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

    /// Normalize IR expressions.
    ///
    /// Normalize IR expressions as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// x: int = 1 + my_node(s, v*2).o;
    /// ```
    ///
    /// The above example becomes:
    ///
    /// ```GR
    /// x_1: int = v*2;
    /// x_2: int = my_node(s, x_1).o;
    /// x: int = 1 + x_2;
    /// ```
    ///
    /// This example is tested in the following code.
    ///
    /// ```rust
    /// use std::collections::HashSet;
    ///
    /// use grustine::common::{constant::Constant, location::Location, scope::Scope, type_system::Type};
    /// use grustine::ir::{
    ///     equation::Equation, expression::Expression, identifier_creator::IdentifierCreator,
    ///     stream_expression::StreamExpression,
    /// };
    ///
    /// let mut identifier_creator = IdentifierCreator {
    ///     signals: HashSet::from([String::from("x"), String::from("s"), String::from("v")]),
    /// };
    /// let mut expression = StreamExpression::MapApplication {
    ///     function_expression: Expression::Call {
    ///         id: String::from("+"),
    ///         typing: Type::Abstract(
    ///             Box::new(Type::Integer),
    ///             Box::new(Type::Abstract(
    ///                 Box::new(Type::Integer),
    ///                 Box::new(Type::Integer),
    ///             )),
    ///         ),
    ///         location: Location::default(),
    ///     },
    ///     inputs: vec![
    ///         StreamExpression::Constant {
    ///             constant: Constant::Integer(1),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         },
    ///         StreamExpression::NodeApplication {
    ///             node: String::from("my_node"),
    ///             inputs: vec![
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 StreamExpression::MapApplication {
    ///                     function_expression: Expression::Call {
    ///                         id: String::from("*2"),
    ///                         typing: Type::Abstract(
    ///                             Box::new(Type::Integer),
    ///                             Box::new(Type::Integer),
    ///                         ),
    ///                         location: Location::default(),
    ///                     },
    ///                     inputs: vec![StreamExpression::SignalCall {
    ///                         id: String::from("v"),
    ///                         typing: Type::Integer,
    ///                         location: Location::default(),
    ///                     }],
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///             ],
    ///             signal: String::from("o"),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         },
    ///     ],
    ///     typing: Type::Integer,
    ///     location: Location::default(),
    /// };
    /// let equations = expression.normalize(&mut identifier_creator);
    ///
    /// let control = vec![
    ///     Equation {
    ///         scope: Scope::Local,
    ///         id: String::from("x_1"),
    ///         signal_type: Type::Integer,
    ///         expression: StreamExpression::MapApplication {
    ///             function_expression: Expression::Call {
    ///                 id: String::from("*2"),
    ///                 typing: Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer)),
    ///                 location: Location::default(),
    ///             },
    ///             inputs: vec![StreamExpression::SignalCall {
    ///                 id: String::from("v"),
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///             }],
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         },
    ///         location: Location::default(),
    ///     },
    ///     Equation {
    ///         scope: Scope::Local,
    ///         id: String::from("x_2"),
    ///         signal_type: Type::Integer,
    ///         expression: StreamExpression::NodeApplication {
    ///             node: String::from("my_node"),
    ///             inputs: vec![
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 StreamExpression::SignalCall {
    ///                     id: String::from("x_1"),
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///             ],
    ///             signal: String::from("o"),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         },
    ///         location: Location::default(),
    ///     },
    /// ];
    /// assert_eq!(equations, control);
    ///
    /// let control = StreamExpression::MapApplication {
    ///     function_expression: Expression::Call {
    ///         id: String::from("+"),
    ///         typing: Type::Abstract(
    ///             Box::new(Type::Integer),
    ///             Box::new(Type::Abstract(
    ///                 Box::new(Type::Integer),
    ///                 Box::new(Type::Integer),
    ///             )),
    ///         ),
    ///         location: Location::default(),
    ///     },
    ///     inputs: vec![
    ///         StreamExpression::Constant {
    ///             constant: Constant::Integer(1),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         },
    ///         StreamExpression::SignalCall {
    ///             id: String::from("x_2"),
    ///             typing: Type::Integer,
    ///             location: Location::default(),
    ///         },
    ///     ],
    ///     typing: Type::Integer,
    ///     location: Location::default(),
    /// };
    /// assert_eq!(expression, control)
    /// ```
    pub fn normalize(&mut self, identifier_creator: &mut IdentifierCreator) -> Vec<Equation> {
        self.normalize_root(identifier_creator)
    }

    fn normalize_root(&mut self, identifier_creator: &mut IdentifierCreator) -> Vec<Equation> {
        match self {
            StreamExpression::FollowedBy { expression, .. } => {
                expression.normalize_cascade(identifier_creator)
            }
            StreamExpression::MapApplication { inputs, .. } => inputs
                .iter_mut()
                .flat_map(|expression| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::NodeApplication { inputs, .. } => inputs
                .iter_mut()
                .flat_map(|expression| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::Structure { fields, .. } => fields
                .iter_mut()
                .flat_map(|(_, expression)| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::Array { elements, .. } => elements
                .iter_mut()
                .flat_map(|expression| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::Match {
                expression, arms, ..
            } => {
                let mut equations = expression.normalize_cascade(identifier_creator);
                arms.iter_mut().for_each(|(_, option, body, expression)| {
                    let mut option_equations = option.as_mut().map_or(vec![], |option| {
                        option.normalize_cascade(identifier_creator)
                    });
                    equations.append(&mut option_equations);

                    let mut expression_equations = expression.normalize_cascade(identifier_creator);
                    body.append(&mut expression_equations)
                });
                equations
            }
            StreamExpression::When {
                option,
                present_body,
                present,
                default_body,
                default,
                ..
            } => {
                let mut present_equations = present.normalize_cascade(identifier_creator);
                present_body.append(&mut present_equations);

                let mut default_equations = default.normalize_cascade(identifier_creator);
                default_body.append(&mut default_equations);

                option.normalize_cascade(identifier_creator)
            }
            _ => vec![],
        }
    }

    fn normalize_cascade(&mut self, identifier_creator: &mut IdentifierCreator) -> Vec<Equation> {
        match self {
            StreamExpression::FollowedBy { expression, .. } => {
                expression.normalize_cascade(identifier_creator)
            }
            StreamExpression::MapApplication { inputs, .. } => inputs
                .iter_mut()
                .flat_map(|expression| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::Structure { fields, .. } => fields
                .iter_mut()
                .flat_map(|(_, expression)| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::Array { elements, .. } => elements
                .iter_mut()
                .flat_map(|expression| expression.normalize_cascade(identifier_creator))
                .collect(),
            StreamExpression::Match {
                expression, arms, ..
            } => {
                let mut equations = expression.normalize_cascade(identifier_creator);
                arms.iter_mut().for_each(|(_, option, body, expression)| {
                    let mut option_equations = option.as_mut().map_or(vec![], |option| {
                        option.normalize_cascade(identifier_creator)
                    });
                    equations.append(&mut option_equations);

                    let mut expression_equations = expression.normalize_cascade(identifier_creator);
                    body.append(&mut expression_equations)
                });
                equations
            }
            StreamExpression::When {
                option,
                present_body,
                present,
                default_body,
                default,
                ..
            } => {
                let mut present_equations = present.normalize_cascade(identifier_creator);
                present_body.append(&mut present_equations);

                let mut default_equations = default.normalize_cascade(identifier_creator);
                default_body.append(&mut default_equations);

                option.normalize_cascade(identifier_creator)
            }
            StreamExpression::NodeApplication {
                node,
                inputs,
                signal,
                typing,
                location,
            } => {
                let mut equations = inputs
                    .iter_mut()
                    .flat_map(|expression| expression.normalize_to_signal_call(identifier_creator))
                    .collect::<Vec<_>>();

                let fresh_id = identifier_creator.new_identifier(
                    String::from("x"),
                    String::from(""),
                    String::from(""),
                );

                let node_application_equation = Equation {
                    scope: Scope::Local,
                    signal_type: typing.clone(),
                    location: location.clone(),
                    expression: StreamExpression::NodeApplication {
                        node: node.clone(),
                        inputs: inputs.clone(),
                        signal: signal.clone(),
                        typing: typing.clone(),
                        location: location.clone(),
                    },
                    id: fresh_id.clone(),
                };

                *self = StreamExpression::SignalCall {
                    id: fresh_id.clone(),
                    typing: typing.clone(),
                    location: location.clone(),
                };

                equations.push(node_application_equation);

                equations
            }
            _ => vec![],
        }
    }

    fn normalize_to_signal_call(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
    ) -> Vec<Equation> {
        match self {
            StreamExpression::SignalCall { .. } => vec![],
            _ => {
                let mut equations = self.normalize_cascade(identifier_creator);

                let typing = self.get_type().clone();
                let location = self.get_location().clone();

                let fresh_id = identifier_creator.new_identifier(
                    String::from("x"),
                    String::from(""),
                    String::from(""),
                );

                let new_equation = Equation {
                    scope: Scope::Local,
                    signal_type: typing.clone(),
                    location: location.clone(),
                    expression: self.clone(),
                    id: fresh_id.clone(),
                };

                *self = StreamExpression::SignalCall {
                    id: fresh_id.clone(),
                    typing: typing,
                    location: location,
                };

                equations.push(new_equation);
                equations
            }
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
                    vec![],
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
                    vec![],
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
                    vec![],
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
                    vec![],
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
            present_body: vec![],
            present: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(0),
                typing: Type::Integer,
                location: Location::default(),
            }),
            default_body: vec![],
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
            present_body: vec![],
            present: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Type::Integer,
                location: Location::default(),
            }),
            default_body: vec![],
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

#[cfg(test)]
mod normalize_to_signal_call {
    use std::collections::HashSet;

    use crate::common::{constant::Constant, location::Location, scope::Scope, type_system::Type};
    use crate::ir::{
        equation::Equation, identifier_creator::IdentifierCreator,
        stream_expression::StreamExpression,
    };

    #[test]
    fn should_leave_signal_call_unchanged() {
        let mut identifier_creator = IdentifierCreator {
            signals: HashSet::new(),
        };
        let mut expression = StreamExpression::SignalCall {
            id: String::from("x"),
            typing: Type::Integer,
            location: Location::default(),
        };
        let equations = expression.normalize_to_signal_call(&mut identifier_creator);

        let control = StreamExpression::SignalCall {
            id: String::from("x"),
            typing: Type::Integer,
            location: Location::default(),
        };
        assert!(equations.is_empty());
        assert_eq!(expression, control)
    }

    #[test]
    fn should_create_signal_call_from_other_expression() {
        let mut identifier_creator = IdentifierCreator {
            signals: HashSet::from([String::from("x")]),
        };
        let mut expression = StreamExpression::FollowedBy {
            constant: Constant::Integer(0),
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Type::Integer,
                location: Location::default(),
            }),
            typing: Type::Integer,
            location: Location::default(),
        };
        let equations = expression.normalize_to_signal_call(&mut identifier_creator);

        let control = Equation {
            scope: Scope::Local,
            id: String::from("x_1"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
                constant: Constant::Integer(0),
                expression: Box::new(StreamExpression::SignalCall {
                    id: String::from("x"),
                    typing: Type::Integer,
                    location: Location::default(),
                }),
                typing: Type::Integer,
                location: Location::default(),
            },
            location: Location::default(),
        };
        assert_eq!(equations[0], control);

        let control = StreamExpression::SignalCall {
            id: String::from("x_1"),
            typing: Type::Integer,
            location: Location::default(),
        };
        assert_eq!(expression, control)
    }
}

#[cfg(test)]
mod normalize {
    use std::collections::HashSet;

    use crate::common::{constant::Constant, location::Location, scope::Scope, type_system::Type};
    use crate::ir::{
        equation::Equation, expression::Expression, identifier_creator::IdentifierCreator,
        stream_expression::StreamExpression,
    };

    #[test]
    fn should_normalize_expression_according_to_rules() {
        // x: int = 1 + my_node(s, v*2).o;
        let mut identifier_creator = IdentifierCreator {
            signals: HashSet::from([String::from("x"), String::from("s"), String::from("v")]),
        };
        let mut expression = StreamExpression::MapApplication {
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
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: Type::Integer,
                    location: Location::default(),
                },
                StreamExpression::NodeApplication {
                    node: String::from("my_node"),
                    inputs: vec![
                        StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("*2"),
                                typing: Type::Abstract(
                                    Box::new(Type::Integer),
                                    Box::new(Type::Integer),
                                ),
                                location: Location::default(),
                            },
                            inputs: vec![StreamExpression::SignalCall {
                                id: String::from("v"),
                                typing: Type::Integer,
                                location: Location::default(),
                            }],
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                    ],
                    signal: String::from("o"),
                    typing: Type::Integer,
                    location: Location::default(),
                },
            ],
            typing: Type::Integer,
            location: Location::default(),
        };
        let equations = expression.normalize(&mut identifier_creator);

        // x_1: int = v*2;
        // x_2: int = my_node(s, x_1).o;
        let control = vec![
            Equation {
                scope: Scope::Local,
                id: String::from("x_1"),
                signal_type: Type::Integer,
                expression: StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("*2"),
                        typing: Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer)),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("v"),
                        typing: Type::Integer,
                        location: Location::default(),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                },
                location: Location::default(),
            },
            Equation {
                scope: Scope::Local,
                id: String::from("x_2"),
                signal_type: Type::Integer,
                expression: StreamExpression::NodeApplication {
                    node: String::from("my_node"),
                    inputs: vec![
                        StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        StreamExpression::SignalCall {
                            id: String::from("x_1"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                    ],
                    signal: String::from("o"),
                    typing: Type::Integer,
                    location: Location::default(),
                },
                location: Location::default(),
            },
        ];
        assert_eq!(equations, control);

        // x: int = 1 + x_2;
        let control = StreamExpression::MapApplication {
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
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: Type::Integer,
                    location: Location::default(),
                },
                StreamExpression::SignalCall {
                    id: String::from("x_2"),
                    typing: Type::Integer,
                    location: Location::default(),
                },
            ],
            typing: Type::Integer,
            location: Location::default(),
        };
        assert_eq!(expression, control)
    }
}
