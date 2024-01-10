use std::collections::HashMap;

use crate::ast::{expression::Expression, typedef::Typedef};
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};

impl Expression {
    /// Add a [Type] to the tuple element access expression.
    pub fn typing_tuple_element_access(
        &mut self,
        global_context: &HashMap<String, Type>,
        elements_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            Expression::TupleElementAccess {
                expression,
                element_number,
                typing,
                location,
            } => {
                expression.typing(global_context, elements_context, user_types_context, errors)?;

                match expression.get_type().unwrap() {
                    Type::Tuple(elements_type) => {
                        let option_element_type = elements_type.get(*element_number);
                        if let Some(element_type) = option_element_type {
                            *typing = Some(element_type.clone());
                            Ok(())
                        } else {
                            let error = Error::IndexOutOfBounds {
                                location: location.clone(),
                            };
                            errors.push(error);
                            Err(TerminationError)
                        }
                    }
                    given_type => {
                        let error = Error::ExpectTuple {
                            given_type: given_type.clone(),
                            location: location.clone(),
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_tuple_element_access {
    use crate::ast::expression::Expression;
    use crate::ast::typedef::Typedef;
    use crate::common::{location::Location, r#type::Type};
    use std::collections::HashMap;

    #[test]
    fn should_type_tuple_element_access() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(
            String::from("p123"),
            Type::Tuple(vec![
                Type::Structure("Point".to_string()),
                Type::Structure("Point".to_string()),
                Type::Structure("Point".to_string()),
            ]),
        );
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

        let mut expression = Expression::TupleElementAccess {
            expression: Box::new(Expression::Call {
                id: String::from("p123"),
                typing: None,
                location: Location::default(),
            }),
            element_number: 2,
            typing: None,
            location: Location::default(),
        };
        let control = Expression::TupleElementAccess {
            expression: Box::new(Expression::Call {
                id: String::from("p123"),
                typing: Some(Type::Tuple(vec![
                    Type::Structure("Point".to_string()),
                    Type::Structure("Point".to_string()),
                    Type::Structure("Point".to_string()),
                ])),
                location: Location::default(),
            }),
            element_number: 2,
            typing: Some(Type::Structure("Point".to_string())),
            location: Location::default(),
        };

        expression
            .typing_tuple_element_access(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_when_expression_not_tuple() {
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

        let mut expression = Expression::TupleElementAccess {
            expression: Box::new(Expression::Call {
                id: String::from("p"),
                typing: None,
                location: Location::default(),
            }),
            element_number: 2,
            typing: None,
            location: Location::default(),
        };

        expression
            .typing_tuple_element_access(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_for_index_out_of_bounds() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(
            String::from("p123"),
            Type::Tuple(vec![
                Type::Structure("Point".to_string()),
                Type::Structure("Point".to_string()),
                Type::Structure("Point".to_string()),
            ]),
        );
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

        let mut expression = Expression::TupleElementAccess {
            expression: Box::new(Expression::Call {
                id: String::from("p123"),
                typing: None,
                location: Location::default(),
            }),
            element_number: 3,
            typing: None,
            location: Location::default(),
        };

        expression
            .typing_tuple_element_access(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}
