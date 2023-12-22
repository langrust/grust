use std::collections::HashMap;

use crate::ast::{pattern::Pattern, typedef::Typedef};
use crate::common::{constant::Constant, location::Location, r#type::Type};
use crate::error::{Error, TerminationError};

mod abstraction;
mod application;
mod array;
mod call;
mod constant;
mod field_access;
mod map;
mod r#match;
mod structure;
mod when;

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust expression AST.
pub enum Expression {
    /// Constant expression.
    Constant {
        /// The constant.
        constant: Constant,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Call expression.
    Call {
        /// Element identifier.
        id: String,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Application expression.
    Application {
        /// The expression applied.
        function_expression: Box<Expression>,
        /// The inputs to the expression.
        inputs: Vec<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Abstraction expression.
    Abstraction {
        /// The inputs to the abstraction.
        inputs: Vec<String>,
        /// The expression abstracted.
        expression: Box<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Abstraction expression with inputs types.
    TypedAbstraction {
        /// The inputs to the abstraction.
        inputs: Vec<(String, Type)>,
        /// The expression abstracted.
        expression: Box<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Structure expression.
    Structure {
        /// The structure name.
        name: String,
        /// The fields associated with their expressions.
        fields: Vec<(String, Expression)>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Array expression.
    Array {
        /// The elements inside the array.
        elements: Vec<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Pattern matching expression.
    Match {
        /// The expression to match.
        expression: Box<Expression>,
        /// The different matching cases.
        arms: Vec<(Pattern, Option<Expression>, Expression)>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// When present expression.
    When {
        /// The identifier of the value when present
        id: String,
        /// The optional expression.
        option: Box<Expression>,
        /// The expression when present.
        present: Box<Expression>,
        /// The default expression.
        default: Box<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Field access expression.
    FieldAccess {
        /// The structure expression.
        expression: Box<Expression>,
        /// The field to access.
        field: String,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Array map operator expression.
    Map {
        /// The array expression.
        expression: Box<Expression>,
        /// The function expression.
        function_expression: Box<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
}

impl Expression {
    /// Add a [Type] to the expression.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::expression::Expression;
    /// use grustine::common::{constant::Constant, location::Location};
    ///
    /// let mut errors = vec![];
    /// let global_context = HashMap::new();
    /// let elements_context = HashMap::new();
    /// let user_types_context = HashMap::new();
    /// let mut expression = Expression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: None,
    ///     location: Location::default(),
    /// };
    /// expression.typing(&global_context, &elements_context, &user_types_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        global_context: &HashMap<String, Type>,
        elements_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            Expression::Constant { .. } => self.typing_constant(user_types_context, errors),
            Expression::Call { .. } => self.typing_call(elements_context, errors),
            Expression::Application { .. } => self.typing_application(
                global_context,
                elements_context,
                user_types_context,
                errors,
            ),
            Expression::TypedAbstraction { .. } | Expression::Abstraction { .. } => {
                self.typing_abstraction(global_context, user_types_context, errors)
            }
            Expression::Structure { .. } => {
                self.typing_structure(global_context, elements_context, user_types_context, errors)
            }
            Expression::Array { .. } => {
                self.typing_array(global_context, elements_context, user_types_context, errors)
            }
            Expression::When { .. } => {
                self.typing_when(global_context, elements_context, user_types_context, errors)
            }
            Expression::Match { .. } => {
                self.typing_match(global_context, elements_context, user_types_context, errors)
            }
            Expression::FieldAccess { .. } => self.typing_field_access(
                global_context,
                elements_context,
                user_types_context,
                errors,
            ),
            Expression::Map { .. } => {
                self.typing_map(global_context, elements_context, user_types_context, errors)
            }
        }
    }

    /// Get the reference to the expression's typing.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::expression::Expression;
    /// use grustine::common::{constant::Constant, location::Location, r#type::Type};
    ///
    /// let mut expression = Expression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Some(Type::Integer),
    ///     location: Location::default(),
    /// };
    /// let typing = expression.get_type().unwrap();
    /// ```
    pub fn get_type(&self) -> Option<&Type> {
        match self {
            Expression::Constant { typing, .. }
            | Expression::Call { typing, .. }
            | Expression::Application { typing, .. }
            | Expression::Abstraction { typing, .. }
            | Expression::TypedAbstraction { typing, .. }
            | Expression::Structure { typing, .. }
            | Expression::Array { typing, .. }
            | Expression::Match { typing, .. }
            | Expression::When { typing, .. }
            | Expression::FieldAccess { typing, .. }
            | Expression::Map { typing, .. } => typing.as_ref(),
        }
    }

    /// Get the mutable reference to the expression's typing.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::expression::Expression;
    /// use grustine::common::{constant::Constant, location::Location, r#type::Type};
    ///
    /// let mut expression = Expression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Some(Type::Integer),
    ///     location: Location::default(),
    /// };
    /// let typing = expression.get_type_mut().unwrap();
    /// ```
    pub fn get_type_mut(&mut self) -> Option<&mut Type> {
        match self {
            Expression::Constant { typing, .. }
            | Expression::Call { typing, .. }
            | Expression::Application { typing, .. }
            | Expression::Abstraction { typing, .. }
            | Expression::TypedAbstraction { typing, .. }
            | Expression::Structure { typing, .. }
            | Expression::Array { typing, .. }
            | Expression::Match { typing, .. }
            | Expression::When { typing, .. }
            | Expression::FieldAccess { typing, .. }
            | Expression::Map { typing, .. } => typing.as_mut(),
        }
    }

    /// Get the expression's typing.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::expression::Expression;
    /// use grustine::common::{constant::Constant, location::Location, r#type::Type};
    ///
    /// let mut expression = Expression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Some(Type::Integer),
    ///     location: Location::default(),
    /// };
    /// let typing = expression.get_type_owned().unwrap();
    /// ```
    pub fn get_type_owned(self) -> Option<Type> {
        match self {
            Expression::Constant { typing, .. }
            | Expression::Call { typing, .. }
            | Expression::Application { typing, .. }
            | Expression::Abstraction { typing, .. }
            | Expression::TypedAbstraction { typing, .. }
            | Expression::Structure { typing, .. }
            | Expression::Array { typing, .. }
            | Expression::Match { typing, .. }
            | Expression::When { typing, .. }
            | Expression::FieldAccess { typing, .. }
            | Expression::Map { typing, .. } => typing,
        }
    }
}

#[cfg(test)]
mod typing {
    use crate::ast::{expression::Expression, typedef::Typedef};
    use crate::common::{constant::Constant, location::Location, pattern::Pattern, r#type::Type};
    use crate::error::Error;
    use std::collections::HashMap;

    #[test]
    fn should_type_constant_expression() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let elements_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut expression = Expression::Constant {
            constant: Constant::Integer(0),
            typing: None,
            location: Location::default(),
        };
        let control = Expression::Constant {
            constant: Constant::Integer(0),
            typing: Some(Constant::Integer(0).get_type()),
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_type_call_expression() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("x"), Type::Integer);
        let user_types_context = HashMap::new();

        let mut expression = Expression::Call {
            id: String::from("x"),
            typing: None,
            location: Location::default(),
        };
        let control = Expression::Call {
            id: String::from("x"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_unknown_element_call() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("x"), Type::Integer);
        let user_types_context = HashMap::new();

        let mut expression = Expression::Call {
            id: String::from("y"),
            typing: None,
            location: Location::default(),
        };
        let control = vec![Error::UnknownElement {
            name: String::from("y"),
            location: Location::default(),
        }];

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();

        assert_eq!(errors, control);
    }

    #[test]
    fn should_type_application_expression() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
        );
        elements_context.insert(String::from("x"), Type::Integer);
        let user_types_context = HashMap::new();

        let mut expression = Expression::Application {
            function_expression: Box::new(Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            }),
            inputs: vec![Expression::Call {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }],
            typing: None,
            location: Location::default(),
        };
        let control = Expression::Application {
            function_expression: Box::new(Expression::Call {
                id: String::from("f"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                location: Location::default(),
            }),
            inputs: vec![Expression::Call {
                id: String::from("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }],
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_incompatible_application() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Float], Box::new(Type::Integer)),
        );
        elements_context.insert(String::from("x"), Type::Integer);
        let user_types_context = HashMap::new();

        let mut expression = Expression::Application {
            function_expression: Box::new(Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            }),
            inputs: vec![Expression::Call {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }],
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_type_abstraction_expression() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let elements_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut expression = Expression::TypedAbstraction {
            inputs: vec![(String::from("x"), Type::Integer)],
            expression: Box::new(Expression::Call {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };
        let control = Expression::TypedAbstraction {
            inputs: vec![(String::from("x"), Type::Integer)],
            expression: Box::new(Expression::Call {
                id: String::from("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_already_defined_input_name() {
        let mut errors = vec![];
        let mut global_context = HashMap::new();
        global_context.insert(String::from("x"), Type::Float);
        let elements_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut expression = Expression::TypedAbstraction {
            inputs: vec![(String::from("x"), Type::Integer)],
            expression: Box::new(Expression::Call {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_for_untyped_abstraction() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let elements_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut expression = Expression::Abstraction {
            inputs: vec![String::from("x")],
            expression: Box::new(Expression::Call {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_type_array() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
        );
        elements_context.insert(String::from("x"), Type::Integer);
        let user_types_context = HashMap::new();

        let mut expression = Expression::Array {
            elements: vec![
                Expression::Call {
                    id: String::from("x"),
                    typing: None,
                    location: Location::default(),
                },
                Expression::Application {
                    function_expression: Box::new(Expression::Call {
                        id: String::from("f"),
                        typing: None,
                        location: Location::default(),
                    }),
                    inputs: vec![Expression::Call {
                        id: String::from("x"),
                        typing: None,
                        location: Location::default(),
                    }],
                    typing: None,
                    location: Location::default(),
                },
                Expression::Constant {
                    constant: Constant::Integer(1),
                    typing: None,
                    location: Location::default(),
                },
            ],
            typing: None,
            location: Location::default(),
        };
        let control = Expression::Array {
            elements: vec![
                Expression::Call {
                    id: String::from("x"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                Expression::Application {
                    function_expression: Box::new(Expression::Call {
                        id: String::from("f"),
                        typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Integer))),
                        location: Location::default(),
                    }),
                    inputs: vec![Expression::Call {
                        id: String::from("x"),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    }],
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                Expression::Constant {
                    constant: Constant::Integer(1),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
            ],
            typing: Some(Type::Array(Box::new(Type::Integer), 3)),
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_multiple_types_array() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
        );
        elements_context.insert(String::from("x"), Type::Integer);
        let user_types_context = HashMap::new();

        let mut expression = Expression::Array {
            elements: vec![
                Expression::Call {
                    id: String::from("x"),
                    typing: None,
                    location: Location::default(),
                },
                Expression::Application {
                    function_expression: Box::new(Expression::Call {
                        id: String::from("f"),
                        typing: None,
                        location: Location::default(),
                    }),
                    inputs: vec![Expression::Call {
                        id: String::from("x"),
                        typing: None,
                        location: Location::default(),
                    }],
                    typing: None,
                    location: Location::default(),
                },
                Expression::Constant {
                    constant: Constant::Float(1.0),
                    typing: None,
                    location: Location::default(),
                },
            ],
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_type_when_expression() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("x"), Type::Option(Box::new(Type::Integer)));
        let user_types_context = HashMap::new();

        let mut expression = Expression::When {
            id: String::from("x"),
            option: Box::new(Expression::Call {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            present: Box::new(Expression::Call {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            default: Box::new(Expression::Constant {
                constant: Constant::Integer(1),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };
        let control = Expression::When {
            id: String::from("x"),
            option: Box::new(Expression::Call {
                id: String::from("x"),
                typing: Some(Type::Option(Box::new(Type::Integer))),
                location: Location::default(),
            }),
            present: Box::new(Expression::Call {
                id: String::from("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            default: Box::new(Expression::Constant {
                constant: Constant::Integer(1),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_incompatible_when() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("x"), Type::Option(Box::new(Type::Integer)));
        let user_types_context = HashMap::new();

        let mut expression = Expression::When {
            id: String::from("x"),
            option: Box::new(Expression::Call {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            present: Box::new(Expression::Call {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            default: Box::new(Expression::Constant {
                constant: Constant::Float(1.0),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_type_structure_expression() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let elements_context = HashMap::new();
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

        let mut expression = Expression::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    Expression::Constant {
                        constant: Constant::Integer(1),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    Expression::Constant {
                        constant: Constant::Integer(2),
                        typing: None,
                        location: Location::default(),
                    },
                ),
            ],
            typing: None,
            location: Location::default(),
        };
        let control = Expression::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    Expression::Constant {
                        constant: Constant::Integer(1),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    Expression::Constant {
                        constant: Constant::Integer(2),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                ),
            ],
            typing: Some(Type::Structure(String::from("Point"))),
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_additionnal_field_in_structure() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let elements_context = HashMap::new();
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

        let mut expression = Expression::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    Expression::Constant {
                        constant: Constant::Integer(1),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    Expression::Constant {
                        constant: Constant::Integer(2),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("z"),
                    Expression::Constant {
                        constant: Constant::Integer(0),
                        typing: None,
                        location: Location::default(),
                    },
                ),
            ],
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_for_missing_field_in_structure() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let elements_context = HashMap::new();
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

        let mut expression = Expression::Structure {
            name: String::from("Point"),
            fields: vec![(
                String::from("x"),
                Expression::Constant {
                    constant: Constant::Integer(1),
                    typing: None,
                    location: Location::default(),
                },
            )],
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_for_incompatible_structure() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let elements_context = HashMap::new();
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

        let mut expression = Expression::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    Expression::Constant {
                        constant: Constant::Integer(1),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    Expression::Constant {
                        constant: Constant::Float(2.0),
                        typing: None,
                        location: Location::default(),
                    },
                ),
            ],
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_expect_structure() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let elements_context = HashMap::new();
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

        let mut expression = Expression::Structure {
            name: String::from("Color"),
            fields: vec![
                (
                    String::from("r"),
                    Expression::Constant {
                        constant: Constant::Integer(1),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("g"),
                    Expression::Constant {
                        constant: Constant::Float(2.0),
                        typing: None,
                        location: Location::default(),
                    },
                ),
                (
                    String::from("b"),
                    Expression::Constant {
                        constant: Constant::Float(2.0),
                        typing: None,
                        location: Location::default(),
                    },
                ),
            ],
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_type_match_structure_expression() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("p"), Type::Structure(String::from("Point")));
        elements_context.insert(
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

        let mut expression = Expression::Match {
            expression: Box::new(Expression::Call {
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
                    Expression::Call {
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
                    Expression::Application {
                        function_expression: Box::new(Expression::Call {
                            id: String::from("add_one"),
                            typing: None,
                            location: Location::default(),
                        }),
                        inputs: vec![Expression::Call {
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
        let control = Expression::Match {
            expression: Box::new(Expression::Call {
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
                    Expression::Call {
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
                    Expression::Application {
                        function_expression: Box::new(Expression::Call {
                            id: String::from("add_one"),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        }),
                        inputs: vec![Expression::Call {
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

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_type_field_access() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("p"), Type::Structure("Point".to_string()));
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

        let mut expression = Expression::FieldAccess {
            expression: Box::new(Expression::Call {
                id: String::from("p"),
                typing: None,
                location: Location::default(),
            }),
            field: "x".to_string(),
            typing: None,
            location: Location::default(),
        };
        let control = Expression::FieldAccess {
            expression: Box::new(Expression::Call {
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
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_when_expression_to_field_access_not_structure() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("p"), Type::Integer);
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

        let mut expression = Expression::FieldAccess {
            expression: Box::new(Expression::Call {
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
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_expression_to_field_access_is_enumeration() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("p"), Type::Structure("Point".to_string()));
        let user_types_context = HashMap::from([(
            "Point".to_string(),
            Typedef::Enumeration {
                id: "Point".to_string(),
                elements: vec!["A".to_string(), "B".to_string()],
                location: Location::default(),
            },
        )]);

        let mut expression = Expression::FieldAccess {
            expression: Box::new(Expression::Call {
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
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_for_unknown_field_to_access() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("p"), Type::Structure("Point".to_string()));
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

        let mut expression = Expression::FieldAccess {
            expression: Box::new(Expression::Call {
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
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_type_map() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("a"), Type::Array(Box::new(Type::Integer), 3));
        elements_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Float)),
        );
        let user_types_context = HashMap::new();

        let mut expression = Expression::Map {
            expression: Box::new(Expression::Call {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Box::new(Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };
        let control = Expression::Map {
            expression: Box::new(Expression::Call {
                id: String::from("a"),
                typing: Some(Type::Array(Box::new(Type::Integer), 3)),
                location: Location::default(),
            }),
            function_expression: Box::new(Expression::Call {
                id: String::from("f"),
                typing: Some(Type::Abstract(vec![Type::Integer], Box::new(Type::Float))),
                location: Location::default(),
            }),
            typing: Some(Type::Array(Box::new(Type::Float), 3)),
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_when_expression_mapped_not_array() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("a"), Type::Integer);
        elements_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Float)),
        );
        let user_types_context = HashMap::new();

        let mut expression = Expression::Map {
            expression: Box::new(Expression::Call {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Box::new(Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_mapping_function_not_compatible_with_array_elements() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("a"), Type::Array(Box::new(Type::Boolean), 3));
        elements_context.insert(
            String::from("f"),
            Type::Abstract(vec![Type::Integer], Box::new(Type::Float)),
        );
        let user_types_context = HashMap::new();

        let mut expression = Expression::Map {
            expression: Box::new(Expression::Call {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Box::new(Expression::Call {
                id: String::from("f"),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };

        expression
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}

#[cfg(test)]
mod get_type {
    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location, r#type::Type};

    #[test]
    fn should_return_none_when_no_typing() {
        let expression = Expression::Constant {
            constant: Constant::Integer(0),
            typing: None,
            location: Location::default(),
        };

        let typing = expression.get_type();
        assert!(typing.is_none());
    }

    #[test]
    fn should_return_a_reference_to_the_type_of_typed_expression() {
        let expression_type = Type::Integer;

        let expression = Expression::Constant {
            constant: Constant::Integer(0),
            typing: Some(expression_type.clone()),
            location: Location::default(),
        };

        let typing = expression.get_type().unwrap();
        assert_eq!(*typing, expression_type);
    }
}

#[cfg(test)]
mod get_type_owned {
    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location, r#type::Type};

    #[test]
    fn should_return_none_when_no_typing() {
        let expression = Expression::Constant {
            constant: Constant::Integer(0),
            typing: None,
            location: Location::default(),
        };

        let typing = expression.get_type_owned();
        assert!(typing.is_none());
    }

    #[test]
    fn should_return_the_type_of_typed_expression() {
        let expression_type = Type::Integer;

        let expression = Expression::Constant {
            constant: Constant::Integer(0),
            typing: Some(expression_type.clone()),
            location: Location::default(),
        };

        let typing = expression.get_type_owned().unwrap();
        assert_eq!(typing, expression_type);
    }
}
