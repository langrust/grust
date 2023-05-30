use std::collections::HashMap;

use crate::ast::{
    constant::Constant, location::Location, pattern::Pattern, type_system::Type,
    user_defined_type::UserDefinedType,
};
use crate::common::context::Context;
use crate::error::Error;

#[derive(Debug, PartialEq, Clone)]
/// LanGRust expression AST.
pub enum Expression {
    /// Constant expression.
    Constant {
        /// The constant.
        constant: Constant,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Call expression.
    Call {
        /// Element identifier.
        id: String,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Application expression.
    Application {
        /// The expression applied.
        function_expression: Box<Expression>,
        /// The inputs to the expression.
        inputs: Vec<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Abstraction expression.
    Abstraction {
        /// The inputs to the abstraction.
        inputs: Vec<String>,
        /// The expression abstracted.
        expression: Box<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Abstraction expression with inputs types.
    TypedAbstraction {
        /// The inputs to the abstraction.
        inputs: Vec<(String, Type)>,
        /// The expression abstracted.
        expression: Box<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Structure expression.
    Structure {
        /// The structure name.
        name: String,
        /// The fields associated with their expressions.
        fields: Vec<(String, Expression)>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Array expression.
    Array {
        /// The elements inside the array.
        elements: Vec<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// Pattern matching expression.
    Match {
        /// The expression to match.
        expression: Box<Expression>,
        /// The different matching cases.
        arms: Vec<(Pattern, Option<Expression>, Expression)>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
    /// When present expression.
    When {
        /// The identifier of the value when present
        id: String,
        /// The optional expression.
        option: Box<Expression>,
        /// The expression when present.
        present: Box<Expression>,
        /// The default expression.
        default: Box<Expression>,
        /// Expression type.
        typing: Option<Type>,
        /// Expression location.
        location: Location,
    },
}

impl Expression {
    /// Add a [Type] to the expression.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{constant::Constant, expression::Expression, location::Location};
    /// let mut errors = vec![];
    /// let elements_context = HashMap::new();
    /// let user_types_context = HashMap::new();
    /// let mut expression = Expression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: None,
    ///     location: Location::default(),
    /// };
    /// expression.typing(&elements_context, &user_types_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        elements_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), Error> {
        match self {
            // typing a constant expression consist of getting the type of the constant
            Expression::Constant {
                constant,
                typing,
                location: _,
            } => {
                *typing = Some(constant.get_type());
                Ok(())
            }
            // the type of a call expression in the type of the called element in the context
            Expression::Call {
                id,
                typing,
                location,
            } => {
                let element_type =
                    elements_context.get_element_or_error(id.clone(), location.clone(), errors)?;
                *typing = Some(element_type.clone());
                Ok(())
            }
            // an application expression type is the result of the application
            // of the inputs types to the abstraction/function type
            Expression::Application {
                function_expression,
                inputs,
                typing,
                location,
            } => {
                let test_typing_function_expression =
                    function_expression.typing(elements_context, user_types_context, errors);
                let test_typing_inputs = inputs
                    .into_iter()
                    .map(|input| input.typing(elements_context, user_types_context, errors))
                    .collect::<Vec<Result<(), Error>>>()
                    .into_iter()
                    .collect::<Result<(), Error>>();

                test_typing_function_expression?;
                test_typing_inputs?;

                let application_type = inputs.iter().fold(
                    Ok(function_expression.get_type().unwrap().clone()),
                    |current_typing, input| {
                        let abstraction_type = current_typing.unwrap().clone();
                        let input_type = input.get_type().unwrap().clone();
                        Ok(abstraction_type.apply(input_type, location.clone(), errors)?)
                    },
                )?;

                *typing = Some(application_type);
                Ok(())
            }
            // the type of a typed abstraction is computed by adding inputs to
            // the context and typing the function body expression
            Expression::TypedAbstraction {
                inputs,
                expression,
                typing,
                location,
            } => {
                let mut local_context = elements_context.clone();
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
                    .collect::<Vec<Result<(), Error>>>()
                    .into_iter()
                    .collect::<Result<(), Error>>()?;
                expression.typing(&local_context, user_types_context, errors)?;

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
                errors.push(error.clone());
                Err(error)
            }
            //
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
            // an array is composed of `n` elements of the same type `t` and
            // its type is `[t; n]`
            Expression::Array {
                elements,
                typing,
                location,
            } => {
                elements
                    .into_iter()
                    .map(|element| element.typing(elements_context, user_types_context, errors))
                    .collect::<Vec<Result<(), Error>>>()
                    .into_iter()
                    .collect::<Result<(), Error>>()?;

                let first_type = elements[0].get_type().unwrap();
                elements
                    .iter()
                    .map(|element| {
                        let element_type = element.get_type().unwrap();
                        element_type.eq_check(first_type, location.clone(), errors)
                    })
                    .collect::<Vec<Result<(), Error>>>()
                    .into_iter()
                    .collect::<Result<(), Error>>()?;

                let array_type = Type::Array(Box::new(first_type.clone()), elements.len());

                *typing = Some(array_type);
                Ok(())
            }
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
                option.typing(elements_context, user_types_context, errors)?;

                let option_type = option.get_type().unwrap();
                match option_type {
                    Type::Option(unwraped_type) => {
                        let mut local_context = elements_context.clone();
                        local_context.insert(id.clone(), *unwraped_type.clone());

                        present.typing(&local_context, user_types_context, errors)?;
                        default.typing(elements_context, user_types_context, errors)?;

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
                        errors.push(error.clone());
                        Err(error)
                    }
                }
            }
            // TODO
            Expression::Match {
                expression,
                arms,
                typing,
                location,
            } => {
                expression.typing(elements_context, user_types_context, errors)?;

                arms.into_iter()
                    .map(|(pattern, optional_test_expression, arm_expression)| {
                        // todo: check pattern match the type of expression

                        let optional_test_expression_typing_test = optional_test_expression
                            .as_mut()
                            .map_or(Ok(()), |expression| {
                                expression.typing(elements_context, user_types_context, errors)?;
                                expression.get_type().unwrap().eq_check(
                                    &Type::Boolean,
                                    location.clone(),
                                    errors,
                                )
                            });

                        let arm_expression_typing_test =
                            arm_expression.typing(elements_context, user_types_context, errors);

                        optional_test_expression_typing_test?;
                        arm_expression_typing_test
                    })
                    .collect::<Vec<Result<(), Error>>>()
                    .into_iter()
                    .collect::<Result<(), Error>>()?;

                let first_type = arms[0].2.get_type().unwrap();
                arms.iter()
                    .map(|(_, _, arm_expression)| {
                        let arm_expression_type = arm_expression.get_type().unwrap();
                        arm_expression_type.eq_check(first_type, location.clone(), errors)
                    })
                    .collect::<Vec<Result<(), Error>>>()
                    .into_iter()
                    .collect::<Result<(), Error>>()?;
                
                // todo: patterns should be exhaustive
                todo!()
            }
        }
    }

    /// Get the reference to the expression's typing.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::{constant::Constant, expression::Expression, location::Location, type_system::Type};
    /// let mut expression = Expression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Some(Type::Integer),
    ///     location: Location::default(),
    /// };
    /// let typing = expression.get_type().unwrap();
    /// ```
    pub fn get_type(&self) -> Option<&Type> {
        match self {
            Expression::Constant {
                constant: _,
                typing,
                location: _,
            } => typing.as_ref(),
            Expression::Call {
                id: _,
                typing,
                location: _,
            } => typing.as_ref(),
            Expression::Application {
                function_expression: _,
                inputs: _,
                typing,
                location: _,
            } => typing.as_ref(),
            Expression::Abstraction {
                inputs: _,
                expression: _,
                typing,
                location: _,
            } => typing.as_ref(),
            Expression::TypedAbstraction {
                inputs: _,
                expression: _,
                typing,
                location: _,
            } => typing.as_ref(),
            Expression::Structure {
                name: _,
                fields: _,
                typing,
                location: _,
            } => typing.as_ref(),
            Expression::Array {
                elements: _,
                typing,
                location: _,
            } => typing.as_ref(),
            Expression::Match {
                expression: _,
                arms: _,
                typing,
                location: _,
            } => typing.as_ref(),
            Expression::When {
                id: _,
                option: _,
                present: _,
                default: _,
                typing,
                location: _,
            } => typing.as_ref(),
        }
    }

    /// Get the expression's typing.
    ///
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::{constant::Constant, expression::Expression, location::Location, type_system::Type};
    /// let mut expression = Expression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: Some(Type::Integer),
    ///     location: Location::default(),
    /// };
    /// let typing = expression.get_type_owned().unwrap();
    /// ```
    pub fn get_type_owned(self) -> Option<Type> {
        match self {
            Expression::Constant {
                constant: _,
                typing,
                location: _,
            } => typing,
            Expression::Call {
                id: _,
                typing,
                location: _,
            } => typing,
            Expression::Application {
                function_expression: _,
                inputs: _,
                typing,
                location: _,
            } => typing,
            Expression::Abstraction {
                inputs: _,
                expression: _,
                typing,
                location: _,
            } => typing,
            Expression::TypedAbstraction {
                inputs: _,
                expression: _,
                typing,
                location: _,
            } => typing,
            Expression::Structure {
                name: _,
                fields: _,
                typing,
                location: _,
            } => typing,
            Expression::Array {
                elements: _,
                typing,
                location: _,
            } => typing,
            Expression::Match {
                expression: _,
                arms: _,
                typing,
                location: _,
            } => typing,
            Expression::When {
                id: _,
                option: _,
                present: _,
                default: _,
                typing,
                location: _,
            } => typing,
        }
    }
}

#[cfg(test)]
mod typing {
    use crate::ast::{
        constant::Constant, expression::Expression, location::Location, type_system::Type,
        user_defined_type::UserDefinedType,
    };
    use crate::error::Error;
    use std::collections::HashMap;

    #[test]
    fn should_type_constant_expression() {
        let mut errors = vec![];
        let elements_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut expression = Expression::Constant {
            constant: Constant::Integer(0),
            typing: None,
            location: Location::default(),
        };
        let control = Expression::Constant {
            constant: Constant::Integer(0),
            typing: Some(Constant::Integer(0).get_type()),
            location: Location::default(),
        };

        expression
            .typing(&elements_context, &user_types_context, &mut errors)
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_type_call_expression() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("x"), Type::Integer);
        let user_types_context = HashMap::new();

        let mut expression = Expression::Call {
            id: String::from("x"),
            typing: None,
            location: Location::default(),
        };
        let control = Expression::Call {
            id: String::from("x"),
            typing: Some(Type::Integer),
            location: Location::default(),
        };

        expression
            .typing(&elements_context, &user_types_context, &mut errors)
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_unknown_element_call() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("x"), Type::Integer);
        let user_types_context = HashMap::new();

        let mut expression = Expression::Call {
            id: String::from("y"),
            typing: None,
            location: Location::default(),
        };
        let control = vec![Error::UnknownElement {
            name: String::from("y"),
            location: Location::default(),
        }];

        expression
            .typing(&elements_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, control);
    }

    #[test]
    fn should_type_application_expression() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();
        elements_context.insert(
            String::from("f"),
            Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer)),
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
                typing: Some(Type::Abstract(
                    Box::new(Type::Integer),
                    Box::new(Type::Integer),
                )),
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
            .typing(&elements_context, &user_types_context, &mut errors)
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_incompatible_application() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();
        elements_context.insert(
            String::from("f"),
            Type::Abstract(Box::new(Type::Float), Box::new(Type::Integer)),
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

        let error = expression
            .typing(&elements_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }

    #[test]
    fn should_type_abstraction_expression() {
        let mut errors = vec![];
        let elements_context = HashMap::new();
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
            .typing(&elements_context, &user_types_context, &mut errors)
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_already_defined_input_name() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("x"), Type::Float);
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
            .typing(&elements_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }

    #[test]
    fn should_raise_error_for_untyped_abstraction() {
        let mut errors = vec![];
        let elements_context = HashMap::new();
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
            .typing(&elements_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }

    #[test]
    fn should_type_array() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();
        elements_context.insert(
            String::from("f"),
            Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer)),
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
                        typing: Some(Type::Abstract(
                            Box::new(Type::Integer),
                            Box::new(Type::Integer),
                        )),
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
            .typing(&elements_context, &user_types_context, &mut errors)
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_multiple_types_array() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();
        elements_context.insert(
            String::from("f"),
            Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer)),
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

        let error = expression
            .typing(&elements_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }

    #[test]
    fn should_type_when_expression() {
        let mut errors = vec![];
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
            .typing(&elements_context, &user_types_context, &mut errors)
            .unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_incompatible_when() {
        let mut errors = vec![];
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

        let error = expression
            .typing(&elements_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }

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
            .typing(&elements_context, &user_types_context, &mut errors)
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
            .typing(&elements_context, &user_types_context, &mut errors)
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
            .typing(&elements_context, &user_types_context, &mut errors)
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
            .typing(&elements_context, &user_types_context, &mut errors)
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
            .typing(&elements_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error]);
    }
}

#[cfg(test)]
mod get_type {
    use crate::ast::{
        constant::Constant, expression::Expression, location::Location, type_system::Type,
    };

    #[test]
    fn should_return_none_when_no_typing() {
        let expression = Expression::Constant {
            constant: Constant::Integer(0),
            typing: None,
            location: Location::default(),
        };

        let typing = expression.get_type();
        assert!(typing.is_none());
    }

    #[test]
    fn should_return_a_reference_to_the_type_of_typed_expression() {
        let expression_type = Type::Integer;

        let expression = Expression::Constant {
            constant: Constant::Integer(0),
            typing: Some(expression_type.clone()),
            location: Location::default(),
        };

        let typing = expression.get_type().unwrap();
        assert_eq!(*typing, expression_type);
    }
}

#[cfg(test)]
mod get_type_owned {
    use crate::ast::{
        constant::Constant, expression::Expression, location::Location, type_system::Type,
    };

    #[test]
    fn should_return_none_when_no_typing() {
        let expression = Expression::Constant {
            constant: Constant::Integer(0),
            typing: None,
            location: Location::default(),
        };

        let typing = expression.get_type_owned();
        assert!(typing.is_none());
    }

    #[test]
    fn should_return_the_type_of_typed_expression() {
        let expression_type = Type::Integer;

        let expression = Expression::Constant {
            constant: Constant::Integer(0),
            typing: Some(expression_type.clone()),
            location: Location::default(),
        };

        let typing = expression.get_type_owned().unwrap();
        assert_eq!(typing, expression_type);
    }
}
