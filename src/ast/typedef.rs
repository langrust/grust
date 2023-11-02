use std::collections::HashMap;

use crate::common::context::Context;
use crate::common::{location::Location, r#type::Type};
use crate::error::{Error, TerminationError};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust user defined type AST.
pub enum Typedef {
    /// Represents a structure definition.
    Structure {
        /// The structure's identifier.
        id: String,
        /// The structure's fields: a field has an identifier and a type.
        fields: Vec<(String, Type)>,
        /// Structure location.
        location: Location,
    },
    /// Represents an enumeration definition.
    Enumeration {
        /// The enumeration's identifier.
        id: String,
        /// The enumeration's elements.
        elements: Vec<String>,
        /// Enumeration location.
        location: Location,
    },
    /// Represents an array definition.
    Array {
        /// The array's identifier.
        id: String,
        /// The array's type.
        array_type: Type,
        /// The array's size.
        size: usize,
        /// Array location.
        location: Location,
    },
}

impl Typedef {
    /// Create a [Type] from a [Typedef].
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::typedef::Typedef;
    /// use grustine::common::{location::Location, r#type::Type};
    /// let typedef = Typedef::Structure {
    ///     id: String::from("Point"),
    ///     fields: vec![
    ///         (String::from("x"), Type::Integer),
    ///         (String::from("y"), Type::Integer),
    ///     ],
    ///     location: Location::default(),
    /// };
    /// let new_type = typedef.into_type();
    /// assert_eq!(new_type, Type::Structure(String::from("Point")))
    /// ```
    pub fn into_type(&self) -> Type {
        match self {
            Typedef::Structure {
                id,
                fields: _,
                location: _,
            } => Type::Structure(id.clone()),
            Typedef::Enumeration {
                id,
                elements: _,
                location: _,
            } => Type::Enumeration(id.clone()),
            Typedef::Array {
                id: _,
                array_type,
                size,
                location: _,
            } => Type::Array(Box::new(array_type.clone()), *size),
        }
    }

    /// Check that structure's fields are well-defined.
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::typedef::Typedef;
    /// use grustine::common::{
    ///     constant::Constant, location::Location, r#type::Type,
    /// };
    ///
    /// let mut errors = vec![];
    ///
    /// let typedef = Typedef::Structure {
    ///     id: String::from("Point"),
    ///     fields: vec![
    ///         (String::from("x"), Type::Integer),
    ///         (String::from("y"), Type::Integer),
    ///     ],
    ///     location: Location::default(),
    /// };
    /// typedef.well_defined_structure::<Constant>(
    ///     &vec![
    ///         (String::from("x"), Constant::Integer(1)),
    ///         (String::from("y"), Constant::Integer(2))
    ///     ],
    ///     |constant, field_type, errors| {
    ///         constant.get_type().eq_check(field_type, Location::default(), errors)
    ///     },
    ///     &mut errors
    /// ).unwrap()
    /// ```
    pub fn well_defined_structure<T>(
        &self,
        fields: &[(String, T)],
        mut well_defined_field: impl FnMut(&T, &Type, &mut Vec<Error>) -> Result<(), TerminationError>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            Typedef::Structure {
                id: name,
                fields: structure_fields,
                location,
            } => {
                // convert the structure_fields into an HashMap
                let structure_fields = structure_fields
                    .iter()
                    .map(|(field_id, field_type)| (field_id.clone(), field_type.clone()))
                    .collect::<HashMap<String, Type>>();

                // zip defined fields with the expected type
                let zipped_fields = fields
                    .iter()
                    .map(|(id, expression)| {
                        Ok((
                            expression,
                            structure_fields.get_field_or_error(
                                name,
                                id,
                                location.clone(),
                                errors,
                            )?,
                        ))
                    })
                    .collect::<Vec<Result<_, TerminationError>>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, TerminationError>>()?;

                // check that every field is well-defined
                zipped_fields
                    .into_iter()
                    .map(|(element, field_type)| well_defined_field(element, field_type, errors))
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                // convert the fields into an HashMap defined_fields
                let defined_fields = fields
                    .iter()
                    .map(|(id, _)| id.clone())
                    .collect::<Vec<String>>();

                // check that there are no missing fields
                structure_fields
                    .keys()
                    .map(|id| {
                        if defined_fields.contains(id) {
                            Ok(())
                        } else {
                            let error = Error::MissingField {
                                structure_name: name.clone(),
                                field_name: id.clone(),
                                location: location.clone(),
                            };
                            errors.push(error);
                            Err(TerminationError)
                        }
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()
            }
            _ => unreachable!(),
        }
    }

    /// Determine the type of the equation if undefined
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::typedef::Typedef;
    /// use grustine::common::{location::Location, r#type::Type};
    ///
    /// let mut errors = vec![];
    /// let mut user_types_context = HashMap::new();
    /// user_types_context.insert(
    ///     String::from("Point"),
    ///     Typedef::Structure {
    ///         id: String::from("Point"),
    ///         fields: vec![
    ///             (String::from("x"), Type::Integer),
    ///             (String::from("y"), Type::Integer),
    ///         ],
    ///         location: Location::default(),
    ///     }
    /// );
    ///
    /// let mut user_type = Typedef::Array {
    ///     id: String::from("Trajectory"),
    ///     array_type: Type::NotDefinedYet(String::from("Point")),
    ///     size: 3,
    ///     location: Location::default(),
    /// };
    ///
    /// let control = Typedef::Array {
    ///     id: String::from("Trajectory"),
    ///     array_type: Type::Structure(String::from("Point")),
    ///     size: 3,
    ///     location: Location::default(),
    /// };
    ///
    /// user_type
    ///     .resolve_undefined_types(&user_types_context, &mut errors)
    ///     .unwrap();
    ///
    /// assert_eq!(user_type, control);
    /// ```
    pub fn resolve_undefined_types(
        &mut self,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            Typedef::Structure {
                fields, location, ..
            } => fields
                .iter_mut()
                .map(|(_, field_type)| {
                    field_type.resolve_undefined(location.clone(), user_types_context, errors)
                })
                .collect::<Vec<Result<(), TerminationError>>>()
                .into_iter()
                .collect::<Result<(), TerminationError>>(),
            Typedef::Enumeration { .. } => Ok(()),
            Typedef::Array {
                array_type,
                location,
                ..
            } => array_type.resolve_undefined(location.clone(), user_types_context, errors),
        }
    }
}

#[cfg(test)]
mod into_type {
    use crate::ast::typedef::Typedef;
    use crate::common::{location::Location, r#type::Type};

    #[test]
    fn should_construct_structure_type_for_user_defined_structure() {
        let typedef = Typedef::Structure {
            id: String::from("Point"),
            fields: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            location: Location::default(),
        };

        let new_type = typedef.into_type();
        let control = Type::Structure(String::from("Point"));

        assert_eq!(new_type, control);
    }

    #[test]
    fn should_construct_enumeration_type_for_user_defined_enumeration() {
        let typedef = Typedef::Enumeration {
            id: String::from("Color"),
            elements: vec![
                String::from("Red"),
                String::from("Blue"),
                String::from("Green"),
                String::from("Yellow"),
            ],
            location: Location::default(),
        };

        let new_type = typedef.into_type();
        let control = Type::Enumeration(String::from("Color"));

        assert_eq!(new_type, control);
    }

    #[test]
    fn should_construct_array_type_for_user_defined_array() {
        let typedef = Typedef::Array {
            id: String::from("Matrix_3_3"),
            array_type: Type::Array(Box::new(Type::Integer), 3),
            size: 3,
            location: Location::default(),
        };

        let new_type = typedef.into_type();
        let control = Type::Array(Box::new(Type::Array(Box::new(Type::Integer), 3)), 3);

        assert_eq!(new_type, control);
    }
}

#[cfg(test)]
mod determine_types {
    use std::collections::HashMap;

    use crate::ast::typedef::Typedef;
    use crate::common::{location::Location, r#type::Type};

    #[test]
    fn should_determine_the_type_of_user_type_when_in_types_context() {
        let mut errors = vec![];
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

        let mut user_type = Typedef::Array {
            id: String::from("Trajectory"),
            array_type: Type::NotDefinedYet(String::from("Point")),
            size: 3,
            location: Location::default(),
        };

        let control = Typedef::Array {
            id: String::from("Trajectory"),
            array_type: Type::Structure(String::from("Point")),
            size: 3,
            location: Location::default(),
        };

        user_type
            .resolve_undefined_types(&user_types_context, &mut errors)
            .unwrap();

        assert_eq!(user_type, control);
    }

    #[test]
    fn should_raise_error_when_undefined_type_not_in_types_context() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();

        let mut user_type = Typedef::Array {
            id: String::from("Trajectory"),
            array_type: Type::NotDefinedYet(String::from("Point")),
            size: 3,
            location: Location::default(),
        };

        user_type
            .resolve_undefined_types(&user_types_context, &mut errors)
            .unwrap_err();
    }
}
