use std::collections::HashMap;

use crate::ast::{constant::Constant, location::Location, pattern::Pattern, type_system::Type};
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
        expression: Box<Expression>,
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
        /// Expression location.
        location: Location,
    },
    /// Array expression.
    Array {
        /// The elements inside the array.
        elements: Vec<Expression>,
        /// Expression location.
        location: Location,
    },
    /// Pattern matching expression.
    Match {
        /// The expression to match.
        expression: Box<Expression>,
        /// The different matching cases.
        arms: Vec<(Pattern, Option<Expression>, Expression)>,
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
    /// let mut elements_context = HashMap::new();
    /// let mut expression = Expression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: None,
    ///     location: Location::default(),
    /// };
    /// expression.typing(&mut elements_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        elements_context: &mut HashMap<String, Type>,
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
            },
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
            },
            // an application expression type is the result of the application
            // of the inputs types to the abstraction/function type
            Expression::Application {
                expression,
                inputs,
                typing,
                location,
            } => {
                let test_typing_expression = expression.typing(elements_context, errors);
                let test_typing_inputs = inputs
                    .into_iter()
                    .map(|input| input.typing(elements_context, errors))
                    .collect::<Vec<Result<(), Error>>>()
                    .into_iter()
                    .collect::<Result<(), Error>>();

                test_typing_expression?;
                test_typing_inputs?;

                let application_type = inputs
                    .iter()
                    .fold(
                        Ok(expression.get_type().unwrap().clone()),
                        |current_typing, input| {
                            let abstraction_type = current_typing.unwrap().clone();
                            let input_type = input.get_type().unwrap().clone();
                            Ok(abstraction_type.apply(input_type, location.clone(), errors)?)
                        }
                    )?;
                
                *typing = Some(application_type);
                Ok(())
            },
            // the type of a typed abstraction is computed by adding inputs to
            // the context and typing the function body expression
            Expression::TypedAbstraction {
                inputs,
                expression,
                typing,
                location
            } => {
                let mut local_context = elements_context.clone();
                inputs
                    .iter()
                    .map(|(name, typing)| local_context.insert_unique(name.clone(), typing.clone(), location.clone(), errors))
                    .collect::<Vec<Result<(), Error>>>()
                    .into_iter()
                    .collect::<Result<(), Error>>()?;
                expression.typing(&mut local_context, errors)?;

                let abstraction_type = inputs
                    .iter()
                    .fold(
                        expression.get_type().unwrap().clone(),
                        |current_type, (_, input_type)| {
                            Type::Abstract(Box::new(input_type.clone()), Box::new(current_type))
                        }
                    );
                
                *typing = Some(abstraction_type);
                Ok(())
            },
            _ => Ok(()),
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
            Expression::Constant { constant: _, typing, location: _ } => typing.as_ref(),
            Expression::Call { id: _, typing, location: _ } => typing.as_ref(),
            Expression::Application { expression: _, inputs: _, typing, location: _ } => typing.as_ref(),
            Expression::Abstraction { inputs: _, expression: _, typing, location: _ } => typing.as_ref(),
            Expression::TypedAbstraction { inputs: _, expression: _, typing, location: _ } => typing.as_ref(),
            Expression::Structure { name: _, fields: _, location: _ } => None,
            Expression::Array { elements: _, location: _ } => None,
            Expression::Match { expression: _, arms: _, location: _ } => None,
            Expression::When { id: _, option: _, present: _, default: _, location: _ } => None,
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
            Expression::Constant { constant: _, typing, location: _ } => typing,
            Expression::Call { id: _, typing, location: _ } => typing,
            Expression::Application { expression: _, inputs: _, typing, location: _ } => typing,
            Expression::Abstraction { inputs: _, expression: _, typing, location: _ } => typing,
            Expression::TypedAbstraction { inputs: _, expression: _, typing, location: _ } => typing,
            Expression::Structure { name: _, fields: _, location: _ } => None,
            Expression::Array { elements: _, location: _ } => None,
            Expression::Match { expression: _, arms: _, location: _ } => None,
            Expression::When { id: _, option: _, present: _, default: _, location: _ } => None,
        }
    }
}

#[cfg(test)]
mod typing {
    use crate::ast::{
        constant::Constant, expression::Expression, location::Location, type_system::Type,
    };
    use crate::error::Error;
    use std::collections::HashMap;

    #[test]
    fn should_type_constant_expression() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();

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

        expression.typing(&mut elements_context, &mut errors).unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_type_call_expression() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("x"), Type::Integer);

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

        expression.typing(&mut elements_context, &mut errors).unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_unknown_element_call() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("x"), Type::Integer);

        let mut expression = Expression::Call {
            id: String::from("y"),
            typing: None,
            location: Location::default(),
        };
        let control = vec![
            Error::UnknownElement {
                name: String::from("y"),
                location: Location::default(),
            }
        ];

        expression.typing(&mut elements_context, &mut errors).unwrap_err();

        assert_eq!(errors, control);
    }

    #[test]
    fn should_type_application_expression() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("f"), Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer)));
        elements_context.insert(String::from("x"), Type::Integer);

        let mut expression = Expression::Application {
            expression: Box::new(Expression::Call {
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
            expression: Box::new(Expression::Call {
                id: String::from("f"),
                typing: Some(Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer))),
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

        expression.typing(&mut elements_context, &mut errors).unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_incompatible_application() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("f"), Type::Abstract(Box::new(Type::Float), Box::new(Type::Integer)));
        elements_context.insert(String::from("x"), Type::Integer);

        let mut expression = Expression::Application {
            expression: Box::new(Expression::Call {
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

        let error = expression.typing(&mut elements_context, &mut errors).unwrap_err();

        assert_eq!(errors, vec![error]);
    }

    #[test]
    fn should_type_abstraction_expression() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();

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
            typing: Some(Type::Abstract(Box::new(Type::Integer), Box::new(Type::Integer))),
            location: Location::default(),
        };

        expression.typing(&mut elements_context, &mut errors).unwrap();

        assert_eq!(expression, control);
    }

    #[test]
    fn should_raise_error_for_already_defined_input_name() {
        let mut errors = vec![];
        let mut elements_context = HashMap::new();
        elements_context.insert(String::from("x"), Type::Float);

        
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

        let error = expression.typing(&mut elements_context, &mut errors).unwrap_err();

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
