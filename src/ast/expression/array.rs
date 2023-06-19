use std::collections::HashMap;

use crate::ast::expression::Expression;
use crate::common::{type_system::Type, user_defined_type::UserDefinedType};
use crate::error::Error;

impl Expression {
    /// Add a [Type] to the array expression.
    pub fn typing_array(
        &mut self,
        global_context: &HashMap<String, Type>,
        elements_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        match self {
            // an array is composed of `n` elements of the same type `t` and
            // its type is `[t; n]`
            Expression::Array {
                elements,
                typing,
                location,
            } => {
                elements
                    .into_iter()
                    .map(|element| {
                        element.typing(global_context, elements_context, user_types_context, errors)
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                let first_type = elements[0].get_type().unwrap();
                elements
                    .iter()
                    .map(|element| {
                        let element_type = element.get_type().unwrap();
                        element_type.eq_check(first_type, location.clone(), errors)
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                let array_type = Type::Array(Box::new(first_type.clone()), elements.len());

                *typing = Some(array_type);
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_array {
    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location, type_system::Type};
    use std::collections::HashMap;

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
            .typing_array(
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
            .typing_array(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}
