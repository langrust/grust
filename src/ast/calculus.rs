use std::collections::HashMap;

use crate::ast::{
    expression::Expression, location::Location, type_system::Type,
    user_defined_type::UserDefinedType,
};
use crate::error::Error;

#[derive(Debug, PartialEq, Clone)]
/// LanGRust calculus AST.
pub struct Calculus {
    /// Identifier of the new element.
    pub id: String,
    /// Element type.
    pub element_type: Type,
    /// The expression defining the element.
    pub expression: Expression,
    /// Calculus location.
    pub location: Location,
}

impl Calculus {
    /// Add a [Type] to the equation.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     constant::Constant, calculus::Calculus, location::Location,
    ///     expression::Expression, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    /// let elements_context = HashMap::new();
    /// let user_types_context = HashMap::new();
    ///
    /// let mut expression = Expression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: None,
    ///     location: Location::default(),
    /// };
    /// let mut calculus = Calculus {
    ///     id: String::from("x"),
    ///     element_type: Type::Integer,
    ///     expression: expression,
    ///     location: Location::default(),
    /// };
    ///
    /// calculus.typing(&elements_context, &user_types_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        elements_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), Error> {
        let Calculus {
            element_type,
            expression,
            location,
            ..
        } = self;

        expression.typing(elements_context, user_types_context, errors)?;

        let expression_type = expression.get_type().unwrap();

        expression_type.eq_check(element_type, location.clone(), errors)
    }
}

#[cfg(test)]
mod typing {
    use std::collections::HashMap;

    use crate::ast::{
        calculus::Calculus, constant::Constant, expression::Expression, location::Location,
        type_system::Type,
    };

    #[test]
    fn should_type_well_defined_equation() {
        let mut errors = vec![];
        let elements_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut calculus = Calculus {
            id: String::from("x"),
            element_type: Type::Integer,
            expression: Expression::Constant {
                constant: Constant::Integer(0),
                typing: None,
                location: Location::default(),
            },
            location: Location::default(),
        };

        let control = Calculus {
            id: String::from("x"),
            element_type: Type::Integer,
            expression: Expression::Constant {
                constant: Constant::Integer(0),
                typing: Some(Type::Integer),
                location: Location::default(),
            },
            location: Location::default(),
        };

        calculus
            .typing(&elements_context, &user_types_context, &mut errors)
            .unwrap();

        assert_eq!(calculus, control)
    }

    #[test]
    fn should_raise_error_for_incompatible_type_in_equation() {
        let mut errors = vec![];
        let elements_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut calculus = Calculus {
            id: String::from("x"),
            element_type: Type::Float,
            expression: Expression::Constant {
                constant: Constant::Integer(0),
                typing: None,
                location: Location::default(),
            },
            location: Location::default(),
        };

        let error = calculus
            .typing(&elements_context, &user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error])
    }
}
