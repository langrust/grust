use std::collections::HashMap;

use crate::ast::{expression::Expression, type_system::Type, user_defined_type::UserDefinedType};
use crate::common::context::Context;
use crate::error::Error;

impl Expression {
    /// Add a [Type] to the abstraction expression.
    pub fn typing_abstraction(
        &mut self,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        match self {
            // the type of a typed abstraction is computed by adding inputs to
            // the context and typing the function body expression
            Expression::TypedAbstraction {
                inputs,
                expression,
                typing,
                location,
            } => {
                // create a local context
                let mut local_context = global_context.clone();
                // add inputs in the context
                inputs
                    .iter()
                    .map(|(name, typing)| {
                        local_context.insert_unique(
                            name.clone(),
                            typing.clone(),
                            location.clone(),
                            errors,
                        )
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // type the abstracted expression with the local context
                expression.typing(global_context, &local_context, user_types_context, errors)?;

                // compute abstraction type
                let abstraction_type = inputs.iter().fold(
                    expression.get_type().unwrap().clone(),
                    |current_type, (_, input_type)| {
                        Type::Abstract(Box::new(input_type.clone()), Box::new(current_type))
                    },
                );

                *typing = Some(abstraction_type);
                Ok(())
            }
            // the type of an abstraction can not be infered on its own
            Expression::Abstraction {
                inputs: _,
                expression: _,
                typing: _,
                location,
            } => {
                let error = Error::NoTypeInference {
                    location: location.clone(),
                };
                errors.push(error);
                Err(())
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod typing_abstraction {
    use crate::ast::{expression::Expression, location::Location, type_system::Type};
    use std::collections::HashMap;

    #[test]
    fn should_type_abstraction_expression() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut expression = Expression::TypedAbstraction {
            inputs: vec![(String::from("x"), Type::Integer)],
            expression: Box::new(Expression::Call {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };
        let control = Expression::TypedAbstraction {
            inputs: vec![(String::from("x"), Type::Integer)],
            expression: Box::new(Expression::Call {
                id: String::from("x"),
                typing: Some(Type::Integer),
                location: Location::default(),
            }),
            typing: Some(Type::Abstract(
                Box::new(Type::Integer),
                Box::new(Type::Integer),
            )),
            location: Location::default(),
        };

        expression
            .typing_abstraction(&global_context, &user_types_context, &mut errors)
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_already_defined_input_name() {
        let mut errors = vec![];
        let mut global_context = HashMap::new();
        global_context.insert(String::from("x"), Type::Float);
        let user_types_context = HashMap::new();

        let mut expression = Expression::TypedAbstraction {
            inputs: vec![(String::from("x"), Type::Integer)],
            expression: Box::new(Expression::Call {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };

        let error = expression
            .typing_abstraction(&global_context, &user_types_context, &mut errors)
            .unwrap_err();
    }

    #[test]
    fn should_raise_error_for_untyped_abstraction() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut expression = Expression::Abstraction {
            inputs: vec![String::from("x")],
            expression: Box::new(Expression::Call {
                id: String::from("x"),
                typing: None,
                location: Location::default(),
            }),
            typing: None,
            location: Location::default(),
        };

        let error = expression
            .typing_abstraction(&global_context, &user_types_context, &mut errors)
            .unwrap_err();
    }
}
