use std::collections::HashMap;

use crate::ast::{expression::Expression, typedef::Typedef};
use crate::common::type_system::Type;
use crate::error::Error;

impl Expression {
    /// Add a [Type] to the when expression.
    pub fn typing_when(
        &mut self,
        global_context: &HashMap<String, Type>,
        elements_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        match self {
            // the type of a when expression is the type of both the default and
            // the present expressions
            Expression::When {
                id,
                option,
                present,
                default,
                typing,
                location,
            } => {
                option.typing(global_context, elements_context, user_types_context, errors)?;

                let option_type = option.get_type().unwrap();
                match option_type {
                    Type::Option(unwraped_type) => {
                        let mut local_context = elements_context.clone();
                        local_context.insert(id.clone(), *unwraped_type.clone());

                        present.typing(
                            global_context,
                            &local_context,
                            user_types_context,
                            errors,
                        )?;
                        default.typing(
                            global_context,
                            elements_context,
                            user_types_context,
                            errors,
                        )?;

                        let present_type = present.get_type().unwrap();
                        let default_type = default.get_type().unwrap();

                        *typing = Some(present_type.clone());
                        default_type.eq_check(present_type, location.clone(), errors)
                    }
                    _ => {
                        let error = Error::ExpectOption {
                            given_type: option_type.clone(),
                            location: location.clone(),
                        };
                        errors.push(error);
                        Err(())
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_when {
    use crate::ast::expression::Expression;
    use crate::common::{constant::Constant, location::Location, type_system::Type};
    use std::collections::HashMap;

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
            .typing_when(
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
            .typing_when(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}
