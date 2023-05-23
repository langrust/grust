use std::collections::HashMap;

use crate::ast::{constant::Constant, location::Location, pattern::Pattern, type_system::Type};
use crate::error::Error;

#[derive(Debug, PartialEq, Clone)]
/// LanGRust expression AST.
pub enum Expression {
    /// Constant expression.
    Constant {
        /// The constant.
        constant: Constant,
        /// Expression type.
        ty: Option<Type>,
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
        /// Expression location.
        location: Location,
    },
    /// Abstraction expression.
    Abstraction {
        /// The inputs to the abstraction.
        inputs: Vec<String>,
        /// The expression abstracted.
        expression: Box<Expression>,
        /// Expression location.
        location: Location,
    },
    /// Abstraction expression with inputs types.
    TypedAbstraction {
        /// The inputs to the abstraction.
        inputs: Vec<(String, Type)>,
        /// The expression abstracted.
        expression: Box<Expression>,
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
    ///     ty: None,
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
            Expression::Constant {
                constant,
                ty,
                location: _,
            } => {
                *ty = Some(constant.get_type());
                Ok(())
            },
            Expression::Call {
                id,
                typing,
                location,
            } => {
                match elements_context.get(id) {
                    Some(t) => {
                        *typing = Some(t.clone());
                        Ok(())
                    },
                    None => {
                        let error = Error::UnknownElement {
                            name: id.clone(),
                            location: location.clone()
                        };
                        errors.push(error);
                        Err(
                            Error::UnknownElement {
                                name: id.clone(),
                                location: location.clone()
                            }
                        )
                    },
                }
            },
            _ => Ok(()),
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
            ty: None,
            location: Location::default(),
        };
        let control = Expression::Constant {
            constant: Constant::Integer(0),
            ty: Some(Constant::Integer(0).get_type()),
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
}
