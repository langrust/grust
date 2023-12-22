use std::collections::HashMap;

use crate::ast::{expression::Expression, typedef::Typedef};
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};

impl Expression {
    /// Add a [Type] to the map expression.
    pub fn typing_map(
        &mut self,
        global_context: &HashMap<String, Type>,
        elements_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            Expression::Map {
                expression,
                function_expression,
                typing,
                location,
            } => {
                // type the expression
                expression.typing(global_context, elements_context, user_types_context, errors)?;

                // verify it is an array
                match expression.get_type().unwrap() {
                    Type::Array(element_type, size) => {
                        // type the function expression
                        function_expression.typing(
                            global_context,
                            elements_context,
                            user_types_context,
                            errors,
                        )?;
                        let function_type = function_expression.get_type_mut().unwrap();

                        // apply the function type to the type of array's elements
                        let new_element_type = function_type.apply(
                            vec![*element_type.clone()],
                            location.clone(),
                            errors,
                        )?;

                        *typing = Some(Type::Array(Box::new(new_element_type), *size));
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
mod typing_map {
    use crate::ast::expression::Expression;
    use crate::common::{location::Location, r#type::Type};
    use std::collections::HashMap;

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
            .typing_map(
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
            .typing_map(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_when_function_not_compatible_with_array_elements() {
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
            .typing_map(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}
