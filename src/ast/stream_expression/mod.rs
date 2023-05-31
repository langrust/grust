use std::collections::HashMap;

use crate::ast::{
    constant::Constant, expression::Expression, location::Location, pattern::Pattern,
    type_system::Type, user_defined_type::UserDefinedType,
};
use crate::error::Error;

mod constant;
mod signal_call;
mod structure;

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
    /// use grustine::ast::{constant::Constant, stream_expression::StreamExpression, location::Location};
    /// let mut errors = vec![];
    /// let signals_context = HashMap::new();
    /// let user_types_context = HashMap::new();
    /// let mut stream_expression = StreamExpression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: None,
    ///     location: Location::default(),
    /// };
    /// stream_expression.typing(&signals_context, &user_types_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        signals_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), Error> {
        match self {
            StreamExpression::Constant { .. } => self.typing_constant(),
            StreamExpression::SignalCall { .. } => self.typing_signal_call(signals_context, errors),
            StreamExpression::FollowedBy { .. } => todo!(),
            StreamExpression::MapApplication { .. } => todo!(),
            StreamExpression::NodeApplication { .. } => todo!(),
            StreamExpression::Structure { .. } => {
                self.typing_structure(signals_context, user_types_context, errors)
            }
            StreamExpression::Array { .. } => todo!(),
            StreamExpression::When { .. } => todo!(),
            StreamExpression::Match { .. } => todo!(),
        }
    }

    /// Get the reference to the stream expression's typing.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::{constant::Constant, stream_expression::StreamExpression, location::Location, type_system::Type};
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
    /// use grustine::ast::{constant::Constant, stream_expression::StreamExpression, location::Location, type_system::Type};
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
}

#[cfg(test)]
mod typing {
    use std::collections::HashMap;

    use crate::ast::{
        constant::Constant, location::Location, stream_expression::StreamExpression,
        type_system::Type, user_defined_type::UserDefinedType,
    };
    use crate::error::Error;

    #[test]
    fn should_type_constant_stream_expression() {
        let mut errors = vec![];
        let signals_context = HashMap::new();
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
            .typing(&signals_context, &user_types_context, &mut errors)
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_type_signal_call_stream_expression() {
        let mut errors = vec![];
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
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
            .typing(&signals_context, &user_types_context, &mut errors)
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_raise_error_for_unknown_signal_call() {
        let mut errors = vec![];
        let mut signals_context = HashMap::new();
        signals_context.insert(String::from("x"), Type::Integer);
        let user_types_context = HashMap::new();

        let mut stream_expression = StreamExpression::SignalCall {
            id: String::from("y"),
            typing: None,
            location: Location::default(),
        };
        let control = vec![Error::UnknownElement {
            name: String::from("y"),
            location: Location::default(),
        }];

        stream_expression
            .typing(&signals_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, control);
    }

    #[test]
    fn should_type_structure_stream_expression() {
        let mut errors = vec![];
        let signals_context = HashMap::new();
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
            .typing(&signals_context, &user_types_context, &mut errors)
            .unwrap();

        assert_eq!(stream_expression, control);
    }

    #[test]
    fn should_raise_error_for_additionnal_field_in_structure() {
        let mut errors = vec![];
        let signals_context = HashMap::new();
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

        let error = stream_expression
            .typing(&signals_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }

    #[test]
    fn should_raise_error_for_missing_field_in_structure() {
        let mut errors = vec![];
        let signals_context = HashMap::new();
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

        let error = stream_expression
            .typing(&signals_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }

    #[test]
    fn should_raise_error_for_incompatible_structure() {
        let mut errors = vec![];
        let signals_context = HashMap::new();
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

        let error = stream_expression
            .typing(&signals_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }

    #[test]
    fn should_raise_error_when_expect_structure() {
        let mut errors = vec![];
        let signals_context = HashMap::new();
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

        let error = stream_expression
            .typing(&signals_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }
}

#[cfg(test)]
mod get_type {
    use crate::ast::{
        constant::Constant, location::Location, stream_expression::StreamExpression,
        type_system::Type,
    };

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
    use crate::ast::{
        constant::Constant, location::Location, stream_expression::StreamExpression,
        type_system::Type,
    };

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
