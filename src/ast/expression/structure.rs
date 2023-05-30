use std::collections::HashMap;

use crate::ast::{expression::Expression, type_system::Type, user_defined_type::UserDefinedType};
use crate::common::context::Context;
use crate::error::Error;

impl Expression {
    /// Add a [Type] to the structure expression.
    pub fn typing_structure(
        &mut self,
        elements_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), Error> {
        match self {
            // the type of the structure is the corresponding structure type
            // if fields match their expected types
            Expression::Structure {
                name,
                fields,
                typing,
                location,
            } => {
                let user_type = user_types_context.get_user_type_or_error(
                    name.clone(),
                    location.clone(),
                    errors,
                )?;

                match user_type {
                    UserDefinedType::Structure {
                        id: _,
                        fields: structure_fields,
                        location: _,
                    } => {
                        fields
                            .into_iter()
                            .map(|(_, expression)| {
                                expression.typing(elements_context, user_types_context, errors)
                            })
                            .collect::<Vec<Result<(), Error>>>()
                            .into_iter()
                            .collect::<Result<(), Error>>()?;

                        let structure_fields = structure_fields
                            .iter()
                            .map(|(field_id, field_type)| (field_id.clone(), field_type.clone()))
                            .collect::<HashMap<String, Type>>();

                        fields
                            .iter()
                            .map(|(id, expression)| {
                                let expression_type = expression.get_type().unwrap();
                                let field_type = structure_fields.get_field_or_error(
                                    name.clone(),
                                    id.clone(),
                                    location.clone(),
                                    errors,
                                )?;
                                expression_type.eq_check(field_type, location.clone(), errors)
                            })
                            .collect::<Vec<Result<(), Error>>>()
                            .into_iter()
                            .collect::<Result<(), Error>>()?;

                        let defined_fields = fields
                            .iter()
                            .map(|(id, _)| id.clone())
                            .collect::<Vec<String>>();
                        structure_fields
                            .iter()
                            .map(|(id, _)| {
                                if defined_fields.contains(id) {
                                    Ok(())
                                } else {
                                    let error = Error::MissingField {
                                        structure_name: name.clone(),
                                        field_name: id.clone(),
                                        location: location.clone(),
                                    };
                                    errors.push(error.clone());
                                    Err(error)
                                }
                            })
                            .collect::<Vec<Result<(), Error>>>()
                            .into_iter()
                            .collect::<Result<(), Error>>()?;

                        *typing = Some(Type::Structure(name.clone()));
                        Ok(())
                    }
                    _ => {
                        let error = Error::ExpectStructure {
                            given_type: user_type.into_type(),
                            location: location.clone(),
                        };
                        errors.push(error.clone());
                        Err(error)
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_structure {
    use crate::ast::{
        constant::Constant, expression::Expression, location::Location, type_system::Type,
        user_defined_type::UserDefinedType,
    };
    use std::collections::HashMap;

    #[test]
    fn should_type_structure_expression() {
        let mut errors = vec![];
        let elements_context = HashMap::new();
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
            .typing_structure(&elements_context, &user_types_context, &mut errors)
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_additionnal_field_in_structure() {
        let mut errors = vec![];
        let elements_context = HashMap::new();
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

        let error = expression
            .typing_structure(&elements_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }

    #[test]
    fn should_raise_error_for_missing_field_in_structure() {
        let mut errors = vec![];
        let elements_context = HashMap::new();
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

        let error = expression
            .typing_structure(&elements_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }

    #[test]
    fn should_raise_error_for_incompatible_structure() {
        let mut errors = vec![];
        let elements_context = HashMap::new();
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

        let error = expression
            .typing_structure(&elements_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }

    #[test]
    fn should_raise_error_when_expect_structure() {
        let mut errors = vec![];
        let elements_context = HashMap::new();
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

        let error = expression
            .typing_structure(&elements_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }
}
