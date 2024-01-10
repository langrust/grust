use std::collections::HashMap;

use crate::ast::{
    expression::Expression, node_description::NodeDescription, pattern::Pattern, typedef::Typedef,
};
use crate::common::{constant::Constant, location::Location, r#type::Type};
use crate::error::{Error, TerminationError};

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
mod when;

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust stream expression AST.
pub enum StreamExpression {
    /// Constant stream expression.
    Constant {
        /// The constant.
        constant: Constant,
        /// Stream Expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Signal call stream expression.
    SignalCall {
        /// Signal identifier.
        id: String,
        /// Stream Expression type.
        typing: Option<Type>,
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
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Function application stream expression.
    FunctionApplication {
        /// The expression applied.
        function_expression: Expression,
        /// The inputs to the expression.
        inputs: Vec<StreamExpression>,
        /// Stream Expression type.
        typing: Option<Type>,
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
        typing: Option<Type>,
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
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Array stream expression.
    Array {
        /// The elements inside the array.
        elements: Vec<StreamExpression>,
        /// Stream Expression type.
        typing: Option<Type>,
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
        typing: Option<Type>,
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
        /// Stream expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Field access stream expression.
    FieldAccess {
        /// The structure expression.
        expression: Box<StreamExpression>,
        /// The field to access.
        field: String,
        /// Stream expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Array map operator stream expression.
    Map {
        /// The array stream expression.
        expression: Box<StreamExpression>,
        /// The function expression.
        function_expression: Expression,
        /// Stream expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Array fold operator stream expression.
    Fold {
        /// The array stream expression.
        expression: Box<StreamExpression>,
        /// The initialization stream expression.
        initialization_expression: Box<StreamExpression>,
        /// The function expression.
        function_expression: Expression,
        /// Stream expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Array sort operator stream expression.
    Sort {
        /// The array stream expression.
        expression: Box<StreamExpression>,
        /// The function expression.
        function_expression: Expression,
        /// Stream expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
    /// Arrays zip operator stream expression.
    Zip {
        /// The array stream expressions.
        arrays: Vec<Expression>,
        /// Stream expression type.
        typing: Option<Type>,
        /// Stream expression location.
        location: Location,
    },
}

impl StreamExpression {
    /// Add a [Type] to the stream expression.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::stream_expression::StreamExpression;
    /// use grustine::common::{constant::Constant, location::Location};
    ///
    /// let mut errors = vec![];
    /// let nodes_context = HashMap::new();
    /// let signals_context = HashMap::new();
    /// let global_context = HashMap::new();
    /// let user_types_context = HashMap::new();
    /// let mut stream_expression = StreamExpression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: None,
    ///     location: Location::default(),
    /// };
    /// stream_expression.typing(&nodes_context, &signals_context, &global_context, &user_types_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            StreamExpression::Constant { .. } => self.typing_constant(user_types_context, errors),
            StreamExpression::SignalCall { .. } => self.typing_signal_call(signals_context, errors),
            StreamExpression::FollowedBy { .. } => self.typing_followed_by(
                nodes_context,
                signals_context,
                global_context,
                user_types_context,
                errors,
            ),
            StreamExpression::FunctionApplication { .. } => self.typing_function_application(
                nodes_context,
                signals_context,
                global_context,
                user_types_context,
                errors,
            ),
            StreamExpression::NodeApplication { .. } => self.typing_node_application(
                nodes_context,
                signals_context,
                global_context,
                user_types_context,
                errors,
            ),
            StreamExpression::Structure { .. } => self.typing_structure(
                nodes_context,
                signals_context,
                global_context,
                user_types_context,
                errors,
            ),
            StreamExpression::Array { .. } => self.typing_array(
                nodes_context,
                signals_context,
                global_context,
                user_types_context,
                errors,
            ),
            StreamExpression::When { .. } => self.typing_when(
                nodes_context,
                signals_context,
                global_context,
                user_types_context,
                errors,
            ),
            StreamExpression::Match { .. } => self.typing_match(
                nodes_context,
                signals_context,
                global_context,
                user_types_context,
                errors,
            ),
            StreamExpression::FieldAccess { .. } => self.typing_field_access(
                nodes_context,
                signals_context,
                global_context,
                user_types_context,
                errors,
            ),
            StreamExpression::Map { .. } => self.typing_map(
                nodes_context,
                signals_context,
                global_context,
                user_types_context,
                errors,
            ),
            StreamExpression::Fold { .. } => self.typing_fold(
                nodes_context,
                signals_context,
                global_context,
                user_types_context,
                errors,
            ),
            StreamExpression::Sort { .. } => self.typing_sort(
                nodes_context,
                signals_context,
                global_context,
                user_types_context,
                errors,
            ),
            StreamExpression::Zip {
                arrays,
                typing,
                location,
            } => todo!(),
        }
    }

    /// Get the reference to the stream expression's typing.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::stream_expression::StreamExpression;
    /// use grustine::common::{constant::Constant, location::Location, r#type::Type};
    ///
    /// let mut stream_expression = StreamExpression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Some(Type::Integer),
    ///     location: Location::default(),
    /// };
    /// let typing = stream_expression.get_type().unwrap();
    /// ```
    pub fn get_type(&self) -> Option<&Type> {
        match self {
            StreamExpression::Constant { typing, .. }
            | StreamExpression::SignalCall { typing, .. }
            | StreamExpression::FollowedBy { typing, .. }
            | StreamExpression::FunctionApplication { typing, .. }
            | StreamExpression::NodeApplication { typing, .. }
            | StreamExpression::Structure { typing, .. }
            | StreamExpression::Array { typing, .. }
            | StreamExpression::Match { typing, .. }
            | StreamExpression::When { typing, .. }
            | StreamExpression::FieldAccess { typing, .. }
            | StreamExpression::Map { typing, .. }
            | StreamExpression::Fold { typing, .. }
            | StreamExpression::Sort { typing, .. }
            | StreamExpression::Zip { typing, .. } => typing.as_ref(),
        }
    }

    /// Get the stream expression's typing.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::stream_expression::StreamExpression;
    /// use grustine::common::{constant::Constant, location::Location, r#type::Type};
    ///
    /// let mut stream_expression = StreamExpression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Some(Type::Integer),
    ///     location: Location::default(),
    /// };
    /// let typing = stream_expression.get_type_owned().unwrap();
    /// ```
    pub fn get_type_owned(self) -> Option<Type> {
        match self {
            StreamExpression::Constant { typing, .. }
            | StreamExpression::SignalCall { typing, .. }
            | StreamExpression::FollowedBy { typing, .. }
            | StreamExpression::FunctionApplication { typing, .. }
            | StreamExpression::NodeApplication { typing, .. }
            | StreamExpression::Structure { typing, .. }
            | StreamExpression::Array { typing, .. }
            | StreamExpression::Match { typing, .. }
            | StreamExpression::When { typing, .. }
            | StreamExpression::FieldAccess { typing, .. }
            | StreamExpression::Map { typing, .. }
            | StreamExpression::Fold { typing, .. }
            | StreamExpression::Sort { typing, .. }
            | StreamExpression::Zip { typing, .. } => typing,
        }
    }
}

#[cfg(test)]
mod typing {
    use std::collections::HashMap;

    use crate::ast::{
        expression::Expression, node_description::NodeDescription,
        stream_expression::StreamExpression, typedef::Typedef,
    };
    use crate::common::{constant::Constant, location::Location, pattern::Pattern, r#type::Type};
    use crate::error::Error;

    #[test]
    fn should_type_constant_stream_expression() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let signals_context = HashMap::new();
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Constant {
            constant: Constant::Integer(0),
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::Constant {
            constant: Constant::Integer(0),
            typing: Some(Constant::Integer(0).get_type()),
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_type_signal_call_stream_expression() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::SignalCall {
            id: String::from("x"),
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::SignalCall {
            id: String::from("x"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_raise_error_for_unknown_signal_call() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::SignalCall {
            id: String::from("y"),
            typing: None,
            location: Location::default(),
        };
        let control = vec![Error::UnknownSignal {
            name: String::from("y"),
            location: Location::default(),
        }];

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();

        assert_eq!(errors, control);
    }

    #[test]
    fn should_type_structure_stream_expression() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let signals_context = HashMap::new();
        let global_context = HashMap::new();
        let mut user_types_context = HashMap::new();
        user_types_context.insert(
            String::from("Point"),
            Typedef::Structure {
                id: String::from("Point"),
                fields: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer),
                ],
                location: Location::default(),
            },
        );

        let mut stream_expression = StreamExpression::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(2),
                        typing: None,
                        location: Location::default(),
                    },
                ),
            ],
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(2),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
            ],
            typing: Some(Type::Structure(String::from("Point"))),
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_raise_error_for_additionnal_field_in_structure() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let signals_context = HashMap::new();
        let global_context = HashMap::new();
        let mut user_types_context = HashMap::new();
        user_types_context.insert(
            String::from("Point"),
            Typedef::Structure {
                id: String::from("Point"),
                fields: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer),
                ],
                location: Location::default(),
            },
        );

        let mut stream_expression = StreamExpression::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(2),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("z"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(0),
                        typing: None,
                        location: Location::default(),
                    },
                ),
            ],
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_for_missing_field_in_structure() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let signals_context = HashMap::new();
        let global_context = HashMap::new();
        let mut user_types_context = HashMap::new();
        user_types_context.insert(
            String::from("Point"),
            Typedef::Structure {
                id: String::from("Point"),
                fields: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer),
                ],
                location: Location::default(),
            },
        );

        let mut stream_expression = StreamExpression::Structure {
            name: String::from("Point"),
            fields: vec![(
                String::from("x"),
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: None,
                    location: Location::default(),
                },
            )],
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_for_incompatible_structure() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let signals_context = HashMap::new();
        let global_context = HashMap::new();
        let mut user_types_context = HashMap::new();
        user_types_context.insert(
            String::from("Point"),
            Typedef::Structure {
                id: String::from("Point"),
                fields: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer),
                ],
                location: Location::default(),
            },
        );

        let mut stream_expression = StreamExpression::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    StreamExpression::Constant {
                        constant: Constant::Float(2.0),
                        typing: None,
                        location: Location::default(),
                    },
                ),
            ],
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_expect_structure() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let signals_context = HashMap::new();
        let global_context = HashMap::new();
        let mut user_types_context = HashMap::new();
        user_types_context.insert(
            String::from("Color"),
            Typedef::Enumeration {
                id: String::from("Color"),
                elements: vec![
                    String::from("Yellow"),
                    String::from("Blue"),
                    String::from("Green"),
                    String::from("Red"),
                ],
                location: Location::default(),
            },
        );

        let mut stream_expression = StreamExpression::Structure {
            name: String::from("Color"),
            fields: vec![
                (
                    String::from("r"),
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("g"),
                    StreamExpression::Constant {
                        constant: Constant::Float(2.0),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("b"),
                    StreamExpression::Constant {
                        constant: Constant::Float(2.0),
                        typing: None,
                        location: Location::default(),
                    },
                ),
            ],
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_type_function_application_stream_expression() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::FunctionApplication {
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            },
            inputs: vec![StreamExpression::SignalCall {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }],
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::FunctionApplication {
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            },
            inputs: vec![StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }],
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_raise_error_for_incompatible_function_application() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Float], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::FunctionApplication {
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            },
            inputs: vec![StreamExpression::SignalCall {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }],
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_type_when_expression() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Option(Box::new(Type::Integer)));
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::When {
            id: String::from("x"),
            option: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            present: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            default: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(1),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::When {
            id: String::from("x"),
            option: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Some(Type::Option(Box::new(Type::Integer))),
                location: Location::default(),
            }),
            present: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            default: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(1),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_raise_error_for_incompatible_when() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Option(Box::new(Type::Integer)));
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::When {
            id: String::from("x"),
            option: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            present: Box::new(StreamExpression::SignalCall {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            default: Box::new(StreamExpression::Constant {
                constant: Constant::Float(1.0),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_type_match_structure_stream_expression() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("p"), Type::Structure(String::from("Point")));
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("add_one"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
        );
        let mut user_types_context = HashMap::new();
        user_types_context.insert(
            String::from("Point"),
            Typedef::Structure {
                id: String::from("Point"),
                fields: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer),
                ],
                location: Location::default(),
            },
        );

        let mut stream_expression = StreamExpression::Match {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: None,
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
                        typing: None,
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
                    StreamExpression::FunctionApplication {
                        function_expression: Expression::Call {
                            id: String::from("add_one"),
                            typing: None,
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            id: String::from("y"),
                            typing: None,
                            location: Location::default(),
                        }],
                        typing: None,
                        location: Location::default(),
                    },
                ),
            ],
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::Match {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: Some(Type::Structure(String::from("Point"))),
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
                        typing: Some(Type::Integer),
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
                            id: String::from("y"),
                            typing: Some(Type::Integer),
                            location: Location::default(),
                        }],
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
            ],
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_type_followed_by_stream_expression() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("add_one"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::FollowedBy {
            constant: Constant::Integer(0),
            expression: Box::new(StreamExpression::FunctionApplication {
                function_expression: Expression::Call {
                    id: String::from("add_one"),
                    typing: None,
                    location: Location::default(),
                },
                inputs: vec![StreamExpression::SignalCall {
                    id: String::from("x"),
                    typing: None,
                    location: Location::default(),
                }],
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::FollowedBy {
            constant: Constant::Integer(0),
            expression: Box::new(StreamExpression::FunctionApplication {
                function_expression: Expression::Call {
                    id: String::from("add_one"),
                    typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                    location: Location::default(),
                },
                inputs: vec![StreamExpression::SignalCall {
                    id: String::from("x"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                }],
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_raise_error_for_incompatible_type_in_followed_by() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("add_one"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::FollowedBy {
            constant: Constant::Float(0.0),
            expression: Box::new(StreamExpression::FunctionApplication {
                function_expression: Expression::Call {
                    id: String::from("add_one"),
                    typing: None,
                    location: Location::default(),
                },
                inputs: vec![StreamExpression::SignalCall {
                    id: String::from("x"),
                    typing: None,
                    location: Location::default(),
                }],
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_type_node_application_stream_expression() {
        let mut errors = vec![];
        let mut nodes_context = HashMap::new();
        nodes_context.insert(
            String::from("my_node"),
            NodeDescription {
                is_component: false,
                inputs: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer),
                ],
                outputs: HashMap::from([(String::from("o"), Type::Integer)]),
                locals: HashMap::new(),
            },
        );
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::NodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::FunctionApplication {
                    function_expression: Expression::Call {
                        id: String::from("f"),
                        typing: None,
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: None,
                        location: Location::default(),
                    }],
                    typing: None,
                    location: Location::default(),
                },
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: None,
                    location: Location::default(),
                },
            ],
            signal: String::from("o"),
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::NodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::FunctionApplication {
                    function_expression: Expression::Call {
                        id: String::from("f"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    }],
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
            ],
            signal: String::from("o"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_raise_error_for_component_call() {
        let mut errors = vec![];
        let mut nodes_context = HashMap::new();
        nodes_context.insert(
            String::from("my_component"),
            NodeDescription {
                is_component: true,
                inputs: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer),
                ],
                outputs: HashMap::from([(String::from("o"), Type::Integer)]),
                locals: HashMap::new(),
            },
        );
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::NodeApplication {
            node: String::from("my_component"),
            inputs: vec![
                StreamExpression::FunctionApplication {
                    function_expression: Expression::Call {
                        id: String::from("f"),
                        typing: None,
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: None,
                        location: Location::default(),
                    }],
                    typing: None,
                    location: Location::default(),
                },
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: None,
                    location: Location::default(),
                },
            ],
            signal: String::from("o"),
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_for_incompatible_node_application() {
        let mut errors = vec![];
        let mut nodes_context = HashMap::new();
        nodes_context.insert(
            String::from("my_node"),
            NodeDescription {
                is_component: false,
                inputs: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer),
                ],
                outputs: HashMap::from([(String::from("o"), Type::Integer)]),
                locals: HashMap::new(),
            },
        );
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Float)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::NodeApplication {
            node: String::from("my_node"),
            inputs: vec![
                StreamExpression::FunctionApplication {
                    function_expression: Expression::Call {
                        id: String::from("f"),
                        typing: None,
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("x"),
                        typing: None,
                        location: Location::default(),
                    }],
                    typing: None,
                    location: Location::default(),
                },
                StreamExpression::Constant {
                    constant: Constant::Integer(1),
                    typing: None,
                    location: Location::default(),
                },
            ],
            signal: String::from("o"),
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_type_field_access() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("p"), Type::Structure("Point".to_string()));
        let global_context = HashMap::new();
        let user_types_context = HashMap::from([(
            "Point".to_string(),
            Typedef::Structure {
                id: "Point".to_string(),
                fields: vec![
                    ("x".to_string(), Type::Integer),
                    ("y".to_string(), Type::Integer),
                ],
                location: Location::default(),
            },
        )]);

        let mut expression = StreamExpression::FieldAccess {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: None,
                location: Location::default(),
            }),
            field: "x".to_string(),
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::FieldAccess {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: Some(Type::Structure("Point".to_string())),
                location: Location::default(),
            }),
            field: "x".to_string(),
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_when_expression_to_field_access_not_structure() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("p"), Type::Integer);
        let global_context = HashMap::new();
        let user_types_context = HashMap::from([(
            "Point".to_string(),
            Typedef::Structure {
                id: "Point".to_string(),
                fields: vec![
                    ("x".to_string(), Type::Integer),
                    ("y".to_string(), Type::Integer),
                ],
                location: Location::default(),
            },
        )]);

        let mut expression = StreamExpression::FieldAccess {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: None,
                location: Location::default(),
            }),
            field: "x".to_string(),
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_expression_to_field_access_is_enumeration() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("p"), Type::Structure("Point".to_string()));
        let global_context = HashMap::new();
        let user_types_context = HashMap::from([(
            "Point".to_string(),
            Typedef::Enumeration {
                id: "Point".to_string(),
                elements: vec!["A".to_string(), "B".to_string()],
                location: Location::default(),
            },
        )]);

        let mut expression = StreamExpression::FieldAccess {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: None,
                location: Location::default(),
            }),
            field: "x".to_string(),
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_for_unknown_field_to_acces() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("p"), Type::Structure("Point".to_string()));
        let global_context = HashMap::new();
        let user_types_context = HashMap::from([(
            "Point".to_string(),
            Typedef::Structure {
                id: "Point".to_string(),
                fields: vec![
                    ("x".to_string(), Type::Integer),
                    ("y".to_string(), Type::Integer),
                ],
                location: Location::default(),
            },
        )]);

        let mut expression = StreamExpression::FieldAccess {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("p"),
                typing: None,
                location: Location::default(),
            }),
            field: "z".to_string(),
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_type_map() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Array(Box::new(Type::Integer), 3));
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Float)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Map {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::Map {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: Some(Type::Array(Box::new(Type::Integer), 3)),
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Float))),
                location: Location::default(),
            },
            typing: Some(Type::Array(Box::new(Type::Float), 3)),
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_raise_error_when_mapped_expression_not_array() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Float)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Map {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_mapping_function_not_compatible_with_array_elements() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Array(Box::new(Type::Boolean), 3));
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Float)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Map {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_type_fold() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Array(Box::new(Type::Integer), 3));
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("sum"),
            Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Fold {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            initialization_expression: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(0),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("sum"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::Fold {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: Some(Type::Array(Box::new(Type::Integer), 3)),
                location: Location::default(),
            }),
            initialization_expression: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(0),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("sum"),
                typing: Some(Type::Abstract(
                    vec![Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            },
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_raise_error_when_folded_expression_not_array() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("sum"),
            Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Fold {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            initialization_expression: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(0),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("sum"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_folding_function_not_compatible_with_folding_inputs() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Array(Box::new(Type::Integer), 3));
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("sum"),
            Type::Abstract(vec![Type::Integer, Type::Float], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Fold {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            initialization_expression: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(0),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("sum"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_folding_function_return_type_not_equal_to_initialization() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Array(Box::new(Type::Integer), 3));
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("sum"),
            Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Float)),
        );
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::Fold {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            initialization_expression: Box::new(StreamExpression::Constant {
                constant: Constant::Integer(0),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("sum"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };

        stream_expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_type_sort() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Array(Box::new(Type::Integer), 3));
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("diff"),
            Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut expression = StreamExpression::Sort {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("diff"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };
        let control = StreamExpression::Sort {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: Some(Type::Array(Box::new(Type::Integer), 3)),
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("diff"),
                typing: Some(Type::Abstract(
                    vec![Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            },
            typing: Some(Type::Array(Box::new(Type::Integer), 3)),
            location: Location::default(),
        };

        expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_when_sorted_expression_not_array() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Integer);
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("diff"),
            Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut expression = StreamExpression::Sort {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("diff"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_sorting_function_not_compatible_with_array_elements() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Array(Box::new(Type::Boolean), 3));
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("diff"),
            Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut expression = StreamExpression::Sort {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("diff"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_sorting_function_not_compatible_with_sorting() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("a"), Type::Array(Box::new(Type::Integer), 3));
        let mut global_context = HashMap::new();
        global_context.insert(
            String::from("diff"),
            Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Boolean)),
        );
        let user_types_context = HashMap::new();

        let mut expression = StreamExpression::Sort {
            expression: Box::new(StreamExpression::SignalCall {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Expression::Call {
                id: String::from("diff"),
                typing: None,
                location: Location::default(),
            },
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &nodes_context,
                &signals_context,
                &global_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}

#[cfg(test)]
mod get_type {
    use crate::ast::stream_expression::StreamExpression;
    use crate::common::{constant::Constant, location::Location, r#type::Type};

    #[test]
    fn should_return_none_when_no_typing() {
        let stream_expression = StreamExpression::Constant {
            constant: Constant::Integer(0),
            typing: None,
            location: Location::default(),
        };

        let typing = stream_expression.get_type();
        assert!(typing.is_none());
    }

    #[test]
    fn should_return_a_reference_to_the_type_of_typed_stream_expression() {
        let expression_type = Type::Integer;

        let stream_expression = StreamExpression::Constant {
            constant: Constant::Integer(0),
            typing: Some(expression_type.clone()),
            location: Location::default(),
        };

        let typing = stream_expression.get_type().unwrap();
        assert_eq!(*typing, expression_type);
    }
}

#[cfg(test)]
mod get_type_owned {
    use crate::ast::stream_expression::StreamExpression;
    use crate::common::{constant::Constant, location::Location, r#type::Type};

    #[test]
    fn should_return_none_when_no_typing() {
        let stream_expression = StreamExpression::Constant {
            constant: Constant::Integer(0),
            typing: None,
            location: Location::default(),
        };

        let typing = stream_expression.get_type_owned();
        assert!(typing.is_none());
    }

    #[test]
    fn should_return_the_type_of_typed_stream_expression() {
        let expression_type = Type::Integer;

        let stream_expression = StreamExpression::Constant {
            constant: Constant::Integer(0),
            typing: Some(expression_type.clone()),
            location: Location::default(),
        };

        let typing = stream_expression.get_type_owned().unwrap();
        assert_eq!(typing, expression_type);
    }
}
