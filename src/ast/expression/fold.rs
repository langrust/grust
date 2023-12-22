use std::collections::HashMap;

use crate::ast::{expression::Expression, typedef::Typedef};
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};

impl Expression {
    /// Add a [Type] to the fold expression.
    pub fn typing_fold(
        &mut self,
        global_context: &HashMap<String, Type>,
        elements_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            Expression::Fold {
                expression,
                initialization_expression,
                function_expression,
                typing,
                location,
            } => {
                // type the expression
                expression.typing(global_context, elements_context, user_types_context, errors)?;

                // verify it is an array
                match expression.get_type().unwrap() {
                    Type::Array(element_type, _) => {
                        // type the initialization expression
                        initialization_expression.typing(
                            global_context,
                            elements_context,
                            user_types_context,
                            errors,
                        )?;
                        let initialization_type = initialization_expression.get_type().unwrap();

                        // type the function expression
                        function_expression.typing(
                            global_context,
                            elements_context,
                            user_types_context,
                            errors,
                        )?;
                        let function_type = function_expression.get_type_mut().unwrap();

                        // apply the function type to the type of the initialization and array's elements
                        let new_type = function_type.apply(
                            vec![initialization_type.clone(), *element_type.clone()],
                            location.clone(),
                            errors,
                        )?;

                        // check the new type is equal to the initialization type
                        new_type.eq_check(initialization_type, location.clone(), errors)?;

                        *typing = Some(new_type);
                        Ok(())
                    }
                    given_type => {
                        let error = Error::ExpectArray {
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
mod typing_fold {
    use crate::ast::expression::Expression;
    use crate::common::constant::Constant;
    use crate::common::{location::Location, r#type::Type};
    use std::collections::HashMap;

    #[test]
    fn should_type_fold() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("a"), Type::Array(Box::new(Type::Integer), 3));
        elements_context.insert(
            String::from("sum"),
            Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut expression = Expression::Fold {
            expression: Box::new(Expression::Call {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            initialization_expression: Box::new(Expression::Constant {
                constant: Constant::Integer(0),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Box::new(Expression::Call {
                id: String::from("sum"),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };
        let control = Expression::Fold {
            expression: Box::new(Expression::Call {
                id: String::from("a"),
                typing: Some(Type::Array(Box::new(Type::Integer), 3)),
                location: Location::default(),
            }),
            initialization_expression: Box::new(Expression::Constant {
                constant: Constant::Integer(0),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            function_expression: Box::new(Expression::Call {
                id: String::from("sum"),
                typing: Some(Type::Abstract(
                    vec![Type::Integer, Type::Integer],
                    Box::new(Type::Integer),
                )),
                location: Location::default(),
            }),
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        expression
            .typing_fold(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_when_expression_not_array() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("a"), Type::Integer);
        elements_context.insert(
            String::from("sum"),
            Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut expression = Expression::Fold {
            expression: Box::new(Expression::Call {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            initialization_expression: Box::new(Expression::Constant {
                constant: Constant::Integer(0),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Box::new(Expression::Call {
                id: String::from("sum"),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };

        expression
            .typing_fold(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_function_not_compatible_with_folding_inputs() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("a"), Type::Array(Box::new(Type::Integer), 3));
        elements_context.insert(
            String::from("sum"),
            Type::Abstract(vec![Type::Integer, Type::Float], Box::new(Type::Integer)),
        );
        let user_types_context = HashMap::new();

        let mut expression = Expression::Fold {
            expression: Box::new(Expression::Call {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            initialization_expression: Box::new(Expression::Constant {
                constant: Constant::Integer(0),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Box::new(Expression::Call {
                id: String::from("sum"),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };

        expression
            .typing_fold(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_function_return_type_not_equal_to_initialization() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("a"), Type::Array(Box::new(Type::Integer), 3));
        elements_context.insert(
            String::from("sum"),
            Type::Abstract(vec![Type::Integer, Type::Integer], Box::new(Type::Float)),
        );
        let user_types_context = HashMap::new();

        let mut expression = Expression::Fold {
            expression: Box::new(Expression::Call {
                id: String::from("a"),
                typing: None,
                location: Location::default(),
            }),
            initialization_expression: Box::new(Expression::Constant {
                constant: Constant::Integer(0),
                typing: None,
                location: Location::default(),
            }),
            function_expression: Box::new(Expression::Call {
                id: String::from("sum"),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };

        expression
            .typing_fold(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}
