use std::collections::HashMap;

use crate::ast::expression::Expression;
use crate::common::{type_system::Type, user_defined_type::UserDefinedType};
use crate::error::Error;

impl Expression {
    /// Add a [Type] to the application expression.
    pub fn typing_application(
        &mut self,
        global_context: &HashMap<String, Type>,
        elements_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        match self {
            // an application expression type is the result of the application
            // of the inputs types to the abstraction/function type
            Expression::Application {
                function_expression,
                inputs,
                typing,
                location,
            } => {
                // type the function expression
                let test_typing_function_expression = function_expression.typing(
                    global_context,
                    elements_context,
                    user_types_context,
                    errors,
                );
                // type all inputs
                let test_typing_inputs = inputs
                    .into_iter()
                    .map(|input| {
                        input.typing(global_context, elements_context, user_types_context, errors)
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>();

                // test if there were some errors
                test_typing_function_expression?;
                test_typing_inputs?;

                // compute the application type
                let input_types = inputs
                    .iter()
                    .map(|input| input.get_type().unwrap().clone())
                    .collect();
                let application_type = function_expression.get_type().unwrap().clone().apply(
                    input_types,
                    location.clone(),
                    errors,
                )?;

                *typing = Some(application_type);
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_application {
    use crate::ast::expression::Expression;
    use crate::common::{location::Location, type_system::Type};
    use std::collections::HashMap;

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
            .typing_application(
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
            .typing_application(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}
