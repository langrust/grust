use std::collections::HashMap;

use crate::ast::{expression::Expression, node_description::NodeDescription};
use crate::common::{
    constant::Constant, location::Location, pattern::Pattern, type_system::Type,
    user_defined_type::UserDefinedType,
};
use crate::error::Error;
use crate::ir::stream_expression::StreamExpression as IRStreamExpression;

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
    /// Map application stream expression.
    MapApplication {
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
        /// Stream Expression type.
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
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        match self {
            StreamExpression::Constant { .. } => self.typing_constant(),
            StreamExpression::SignalCall { .. } => self.typing_signal_call(signals_context, errors),
            StreamExpression::FollowedBy { .. } => self.typing_followed_by(
                nodes_context,
                signals_context,
                global_context,
                user_types_context,
                errors,
            ),
            StreamExpression::MapApplication { .. } => self.typing_map_application(
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
        }
    }

    /// Get the reference to the stream expression's typing.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::stream_expression::StreamExpression;
    /// use grustine::common::{constant::Constant, location::Location, type_system::Type};
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
            StreamExpression::Constant { typing, .. } => typing.as_ref(),
            StreamExpression::SignalCall { typing, .. } => typing.as_ref(),
            StreamExpression::FollowedBy { typing, .. } => typing.as_ref(),
            StreamExpression::MapApplication { typing, .. } => typing.as_ref(),
            StreamExpression::NodeApplication { typing, .. } => typing.as_ref(),
            StreamExpression::Structure { typing, .. } => typing.as_ref(),
            StreamExpression::Array { typing, .. } => typing.as_ref(),
            StreamExpression::Match { typing, .. } => typing.as_ref(),
            StreamExpression::When { typing, .. } => typing.as_ref(),
        }
    }

    /// Get the stream expression's typing.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::stream_expression::StreamExpression;
    /// use grustine::common::{constant::Constant, location::Location, type_system::Type};
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
            StreamExpression::Constant { typing, .. } => typing,
            StreamExpression::SignalCall { typing, .. } => typing,
            StreamExpression::FollowedBy { typing, .. } => typing,
            StreamExpression::MapApplication { typing, .. } => typing,
            StreamExpression::NodeApplication { typing, .. } => typing,
            StreamExpression::Structure { typing, .. } => typing,
            StreamExpression::Array { typing, .. } => typing,
            StreamExpression::Match { typing, .. } => typing,
            StreamExpression::When { typing, .. } => typing,
        }
    }

    /// Transform AST stream expressions into IR stream expressions.
    pub fn into_ir(self) -> IRStreamExpression {
        match self {
            StreamExpression::Constant {
                constant,
                typing,
                location,
            } => IRStreamExpression::Constant {
                constant,
                typing: typing.unwrap(),
                location,
            },
            StreamExpression::SignalCall {
                id,
                typing,
                location,
            } => IRStreamExpression::SignalCall {
                id,
                typing: typing.unwrap(),
                location,
            },
            StreamExpression::MapApplication {
                function_expression,
                inputs,
                typing,
                location,
            } => IRStreamExpression::MapApplication {
                function_expression: function_expression.into_ir(),
                inputs: inputs.into_iter().map(|input| input.into_ir()).collect(),
                typing: typing.unwrap(),
                location,
            },
            StreamExpression::Structure {
                name,
                fields,
                typing,
                location,
            } => IRStreamExpression::Structure {
                name,
                fields: fields
                    .into_iter()
                    .map(|(field, expression)| (field, expression.into_ir()))
                    .collect(),
                typing: typing.unwrap(),
                location,
            },
            StreamExpression::Array {
                elements,
                typing,
                location,
            } => IRStreamExpression::Array {
                elements: elements
                    .into_iter()
                    .map(|expression| expression.into_ir())
                    .collect(),
                typing: typing.unwrap(),
                location,
            },
            StreamExpression::Match {
                expression,
                arms,
                typing,
                location,
            } => IRStreamExpression::Match {
                expression: Box::new(expression.into_ir()),
                arms: arms
                    .into_iter()
                    .map(|(pattern, optional_expression, expression)| {
                        (
                            pattern,
                            optional_expression.map(|expression| expression.into_ir()),
                            vec![],
                            expression.into_ir(),
                        )
                    })
                    .collect(),
                typing: typing.unwrap(),
                location,
            },
            StreamExpression::When {
                id,
                option,
                present,
                default,
                typing,
                location,
            } => IRStreamExpression::When {
                id,
                option: Box::new(option.into_ir()),
                present_body: vec![],
                present: Box::new(present.into_ir()),
                default_body: vec![],
                default: Box::new(default.into_ir()),
                typing: typing.unwrap(),
                location,
            },
            StreamExpression::FollowedBy {
                constant,
                expression,
                typing,
                location,
            } => IRStreamExpression::FollowedBy {
                constant,
                expression: Box::new(expression.into_ir()),
                typing: typing.unwrap(),
                location,
            },
            StreamExpression::NodeApplication {
                node,
                inputs,
                signal,
                typing,
                location,
            } => IRStreamExpression::NodeApplication {
                node,
                inputs: inputs.into_iter().map(|input| input.into_ir()).collect(),
                signal,
                typing: typing.unwrap(),
                location,
            },
        }
    }
}

#[cfg(test)]
mod typing {
    use std::collections::HashMap;

    use crate::ast::{
        expression::Expression, node_description::NodeDescription,
        stream_expression::StreamExpression,
    };
    use crate::common::{
        constant::Constant, location::Location, pattern::Pattern, type_system::Type,
        user_defined_type::UserDefinedType,
    };
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
            UserDefinedType::Structure {
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
            UserDefinedType::Structure {
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
            UserDefinedType::Structure {
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
            UserDefinedType::Structure {
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
            UserDefinedType::Enumeration {
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
    fn should_type_map_application_stream_expression() {
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

        let mut stream_expression = StreamExpression::MapApplication {
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
        let control = StreamExpression::MapApplication {
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
    fn should_raise_error_for_incompatible_map_application() {
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

        let mut stream_expression = StreamExpression::MapApplication {
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
            UserDefinedType::Structure {
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
                    StreamExpression::MapApplication {
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
                    StreamExpression::MapApplication {
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
            expression: Box::new(StreamExpression::MapApplication {
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
            expression: Box::new(StreamExpression::MapApplication {
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
            expression: Box::new(StreamExpression::MapApplication {
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
                StreamExpression::MapApplication {
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
                StreamExpression::MapApplication {
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
                StreamExpression::MapApplication {
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
                StreamExpression::MapApplication {
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
}

#[cfg(test)]
mod get_type {
    use crate::ast::stream_expression::StreamExpression;
    use crate::common::{constant::Constant, location::Location, type_system::Type};

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
    use crate::common::{constant::Constant, location::Location, type_system::Type};

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
