use std::collections::HashMap;

use crate::ast::{
    constant::Constant, location::Location, type_system::Type, user_defined_type::UserDefinedType,
};
use crate::common::context::Context;
use crate::error::Error;

use std::fmt::{self, Display};

#[derive(Debug, PartialEq, Clone)]
/// LanGRust matching pattern AST.
pub enum Pattern {
    /// Identifier pattern, gives a name to the matching expression.
    Identifier {
        /// Identifier name.
        name: String,
        /// Pattern location.
        location: Location,
    },
    /// Constant pattern, matches le given constant.
    Constant {
        /// The matching constant.
        constant: Constant,
        /// Pattern location.
        location: Location,
    },
    /// Structure pattern that matches the structure and its fields.
    Structure {
        /// The structure name.
        name: String,
        /// The structure fields with the corresponding patterns to match.
        fields: Vec<(String, Pattern)>,
        /// Pattern location.
        location: Location,
    },
    /// Some pattern that matches when an optional has a value which match the pattern.
    Some {
        /// The pattern matching the value.
        pattern: Box<Pattern>,
        /// Pattern location.
        location: Location,
    },
    /// None pattern, matches when the optional does not have a value.
    None {
        /// Pattern location.
        location: Location,
    },
    /// The default pattern that matches anything.
    Default {
        /// Pattern location.
        location: Location,
    },
}
impl Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Identifier { name, location: _ } => write!(f, "{}", name),
            Pattern::Constant {
                constant,
                location: _,
            } => write!(f, "{}", constant),
            Pattern::Structure {
                name,
                fields,
                location: _,
            } => {
                write!(f, "{} {{ ", name)?;
                for (field, pattern) in fields.iter() {
                    write!(f, "{}: {},", field, pattern)?;
                }
                write!(f, " }}")
            }
            Pattern::Some {
                pattern,
                location: _,
            } => write!(f, "some({})", pattern),
            Pattern::None { location: _ } => write!(f, "none"),
            Pattern::Default { location: _ } => write!(f, "_"),
        }
    }
}

impl Pattern {
    /// Check if `self` pattern matches the expected [Type]
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     constant::Constant, location::Location, pattern::Pattern,
    ///     type_system::Type, user_defined_type::UserDefinedType
    /// };
    ///
    /// let mut errors = vec![];
    /// let mut user_types_context = HashMap::new();
    /// let mut elements_context = HashMap::new();
    ///
    /// user_types_context.insert(
    ///    String::from("Point"),
    ///     UserDefinedType::Structure {
    ///         id: String::from("Point"),
    ///         fields: vec![
    ///             (String::from("x"), Type::Integer),
    ///             (String::from("y"), Type::Integer),
    ///         ],
    ///         location: Location::default(),
    ///     },
    /// );
    ///
    /// let given_pattern = Pattern::Structure {
    ///     name: String::from("Point"),
    ///     fields: vec![
    ///         (
    ///             String::from("x"),
    ///             Pattern::Constant {
    ///                 constant: Constant::Integer(1),
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///         (
    ///             String::from("y"),
    ///             Pattern::Identifier {
    ///                 name: String::from("y"),
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ],
    ///     location: Location::default(),
    /// };
    /// let expected_type = Type::Structure(String::from("Point"));
    ///
    /// given_pattern.construct_context(&expected_type, &mut elements_context, &user_types_context, &mut errors).unwrap();
    ///
    /// let y = String::from("y");
    /// assert_eq!(elements_context[&y], Type::Integer);
    /// assert!(errors.is_empty());
    /// ```
    pub fn construct_context(
        &self,
        expected_type: &Type,
        elements_context: &mut HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        match self {
            Pattern::Identifier { name, location } => elements_context.insert_unique(
                name.clone(),
                expected_type.clone(),
                location.clone(),
                errors,
            ),
            Pattern::Constant { constant, location } => {
                if constant.get_type().eq(expected_type) {
                    Ok(())
                } else {
                    let error = Error::IncompatiblePattern {
                        given_pattern: self.clone(),
                        expected_type: expected_type.clone(),
                        location: location.clone(),
                    };
                    errors.push(error);
                    Err(())
                }
            }
            Pattern::Structure {
                name,
                fields,
                location,
            } => match expected_type {
                Type::Structure(structure_name) if name.eq(structure_name) => {
                    let user_type = user_types_context.get(structure_name).unwrap();
                    match user_type {
                        UserDefinedType::Structure { .. } => user_type.well_defined_structure(
                            fields,
                            |pattern, field_type, errors| {
                                pattern.construct_context(
                                    field_type,
                                    elements_context,
                                    user_types_context,
                                    errors,
                                )
                            },
                            errors,
                        ),
                        _ => unreachable!(),
                    }
                }
                _ => {
                    let error = Error::IncompatiblePattern {
                        given_pattern: self.clone(),
                        expected_type: expected_type.clone(),
                        location: location.clone(),
                    };
                    errors.push(error);
                    Err(())
                }
            },
            Pattern::Some { pattern, location } => match expected_type {
                Type::Option(optional_type) => pattern.construct_context(
                    optional_type,
                    elements_context,
                    user_types_context,
                    errors,
                ),
                _ => {
                    let error = Error::IncompatiblePattern {
                        given_pattern: self.clone(),
                        expected_type: expected_type.clone(),
                        location: location.clone(),
                    };
                    errors.push(error);
                    Err(())
                }
            },
            Pattern::None { location } => match expected_type {
                Type::Option(_) => Ok(()),
                _ => {
                    let error = Error::IncompatiblePattern {
                        given_pattern: self.clone(),
                        expected_type: expected_type.clone(),
                        location: location.clone(),
                    };
                    errors.push(error);
                    Err(())
                }
            },
            Pattern::Default { location: _ } => Ok(()),
        }
    }

    /// Get locally defined signals in patterns.
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::{
    ///     constant::Constant, location::Location, pattern::Pattern
    /// };
    ///
    /// let pattern = Pattern::Structure {
    ///     name: String::from("Point"),
    ///     fields: vec![
    ///         (
    ///             String::from("x"),
    ///             Pattern::Constant {
    ///                 constant: Constant::Integer(1),
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///         (
    ///             String::from("y"),
    ///             Pattern::Identifier {
    ///                 name: String::from("y"),
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ],
    ///     location: Location::default(),
    /// };
    ///
    /// let local_signals = pattern.local_signals();
    /// let control = vec![String::from("y")];
    ///
    /// assert_eq!(local_signals, control);
    /// ```
    pub fn local_signals(&self) -> Vec<String> {
        match self {
            Pattern::Identifier { name, .. } => vec![name.clone()],
            Pattern::Structure { fields, .. } => fields
                .iter()
                .flat_map(|(_, pattern)| pattern.local_signals())
                .collect(),
            Pattern::Some { pattern, .. } => pattern.local_signals(),
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod construct_context {
    use std::collections::HashMap;

    use crate::ast::{
        constant::Constant, location::Location, pattern::Pattern, type_system::Type,
        user_defined_type::UserDefinedType,
    };

    #[test]
    fn should_check_identifier_pattern_for_any_type() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();
        let mut elements_context = HashMap::new();

        let given_pattern = Pattern::Identifier {
            name: String::from("y"),
            location: Location::default(),
        };
        let expected_type = Type::Integer;

        given_pattern
            .construct_context(
                &expected_type,
                &mut elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap()
    }

    #[test]
    fn should_check_constant_pattern_for_constant_type() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();
        let mut elements_context = HashMap::new();
        let given_pattern = Pattern::Constant {
            constant: Constant::Integer(1),
            location: Location::default(),
        };
        let expected_type = Type::Integer;

        given_pattern
            .construct_context(
                &expected_type,
                &mut elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap()
    }

    #[test]
    fn should_raise_error_for_constant_pattern_and_mismatching_type() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();
        let mut elements_context = HashMap::new();
        let given_pattern = Pattern::Constant {
            constant: Constant::Integer(1),
            location: Location::default(),
        };
        let expected_type = Type::Float;

        given_pattern
            .construct_context(
                &expected_type,
                &mut elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_check_structure_pattern_for_structure_type() {
        let mut errors = vec![];
        let mut user_types_context = HashMap::new();
        let mut elements_context = HashMap::new();

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

        let given_pattern = Pattern::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    Pattern::Constant {
                        constant: Constant::Integer(1),
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
        };
        let expected_type = Type::Structure(String::from("Point"));

        given_pattern
            .construct_context(
                &expected_type,
                &mut elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap()
    }

    #[test]
    fn should_raise_error_for_structure_pattern_and_mismatching_type() {
        let mut errors = vec![];
        let mut user_types_context = HashMap::new();
        let mut elements_context = HashMap::new();

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

        let given_pattern = Pattern::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    Pattern::Constant {
                        constant: Constant::Integer(1),
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
        };
        let expected_type = Type::Structure(String::from("Coordinates"));

        given_pattern
            .construct_context(
                &expected_type,
                &mut elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_check_some_pattern_for_option_type() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();
        let mut elements_context = HashMap::new();
        let given_pattern = Pattern::Some {
            pattern: Box::new(Pattern::Identifier {
                name: String::from("y"),
                location: Location::default(),
            }),
            location: Location::default(),
        };
        let expected_type = Type::Option(Box::new(Type::Integer));

        given_pattern
            .construct_context(
                &expected_type,
                &mut elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap()
    }

    #[test]
    fn should_raise_error_for_some_pattern_and_non_option_type() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();
        let mut elements_context = HashMap::new();
        let given_pattern = Pattern::Some {
            pattern: Box::new(Pattern::Identifier {
                name: String::from("y"),
                location: Location::default(),
            }),
            location: Location::default(),
        };
        let expected_type = Type::Integer;

        given_pattern
            .construct_context(
                &expected_type,
                &mut elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_check_none_pattern_for_option_type() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();
        let mut elements_context = HashMap::new();
        let given_pattern = Pattern::None {
            location: Location::default(),
        };
        let expected_type = Type::Option(Box::new(Type::Integer));

        given_pattern
            .construct_context(
                &expected_type,
                &mut elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap()
    }

    #[test]
    fn should_raise_error_for_none_pattern_and_non_option_type() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();
        let mut elements_context = HashMap::new();
        let given_pattern = Pattern::None {
            location: Location::default(),
        };
        let expected_type = Type::Integer;

        given_pattern
            .construct_context(
                &expected_type,
                &mut elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_check_default_pattern_for_any_type() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();
        let mut elements_context = HashMap::new();
        let given_pattern = Pattern::Default {
            location: Location::default(),
        };
        let expected_type = Type::Integer;

        given_pattern
            .construct_context(
                &expected_type,
                &mut elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap()
    }

    #[test]
    fn should_add_local_variables_to_context() {
        let mut errors = vec![];
        let mut user_types_context = HashMap::new();
        let mut elements_context = HashMap::new();

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

        let given_pattern = Pattern::Structure {
            name: String::from("Point"),
            fields: vec![
                (
                    String::from("x"),
                    Pattern::Constant {
                        constant: Constant::Integer(1),
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
        };
        let expected_type = Type::Structure(String::from("Point"));

        given_pattern
            .construct_context(
                &expected_type,
                &mut elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        let y = String::from("y");
        assert_eq!(elements_context[&y], Type::Integer);
        assert!(errors.is_empty());
    }
}
