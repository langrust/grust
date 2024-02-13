use std::collections::HashMap;

use crate::common::{constant::Constant, context::Context, location::Location, r#type::Type};
use crate::error::{Error, TerminationError};
use crate::hir::{pattern::Pattern, typedef::Typedef};

use std::fmt::{self, Display};

impl Pattern {
    /// Check if `self` pattern matches the expected [Type]
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::{pattern::Pattern, typedef::Typedef};
    /// use grustine::common::{constant::Constant, location::Location, r#type::Type};
    ///
    /// let mut errors = vec![];
    /// let mut user_types_context = HashMap::new();
    /// let mut elements_context = HashMap::new();
    ///
    /// user_types_context.insert(
    ///    String::from("Point"),
    ///     Typedef::Structure {
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
    pub fn check_type(
        &self,
        expected_type: &Type,
        symbol_table: &mut SymbolTable,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
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
                    Err(TerminationError)
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
                        Typedef::Structure { .. } => user_type.well_defined_structure(
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
                    Err(TerminationError)
                }
            },
            Pattern::Tuple { elements, location } => match expected_type {
                Type::Tuple(elements_type) if elements.len() == elements_type.len() => elements
                    .iter()
                    .zip(elements_type)
                    .map(|(pattern, element_type)| {
                        pattern.construct_context(
                            element_type,
                            elements_context,
                            user_types_context,
                            errors,
                        )
                    })
                    .collect::<Vec<Result<_, TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>(),
                _ => {
                    let error = Error::IncompatiblePattern {
                        given_pattern: self.clone(),
                        expected_type: expected_type.clone(),
                        location: location.clone(),
                    };
                    errors.push(error);
                    Err(TerminationError)
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
                    Err(TerminationError)
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
                    Err(TerminationError)
                }
            },
            Pattern::Default { location: _ } => Ok(()),
        }
    }

    /// Get locally defined identifiers in patterns.
    ///
    /// # Example
    ///
    /// A pattern of the following form creates a local identifier `y`.
    ///
    /// ```GR
    /// Point {
    ///     x: 1,
    ///     y: y,
    /// }
    /// ```
    ///
    /// This example correspond to the following test.
    ///
    /// ```rust
    /// use grustine::ast::pattern::Pattern;
    /// use grustine::common::{
    ///     constant::Constant, location::Location
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
    /// let local_identifiers = pattern.local_identifiers();
    /// let control = vec![String::from("y")];
    ///
    /// assert_eq!(local_identifiers, control);
    /// ```
    pub fn local_identifiers(&self) -> Vec<String> {
        match self {
            Pattern::Identifier { name, .. } => vec![name.clone()],
            Pattern::Structure { fields, .. } => fields
                .iter()
                .flat_map(|(_, pattern)| pattern.local_identifiers())
                .collect(),
            Pattern::Some { pattern, .. } => pattern.local_identifiers(),
            Pattern::Tuple { elements, .. } => elements
                .iter()
                .flat_map(|pattern| pattern.local_identifiers())
                .collect(),
            Pattern::Constant { .. } | Pattern::None { .. } | Pattern::Default { .. } => vec![],
        }
    }
}
