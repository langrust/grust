use std::collections::HashMap;

use crate::ast::{
    calculus::Calculus, expression::Expression, global_context, location::Location,
    type_system::Type, user_defined_type::UserDefinedType,
};
use crate::common::context::Context;
use crate::error::Error;

#[derive(Debug, PartialEq)]
/// LanGRust function AST.
pub struct Function {
    /// Function identifier.
    pub id: String,
    /// Function's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Function's calculi.
    pub calculi: Vec<(String, Calculus)>,
    /// Function's returned expression and its type.
    pub returned: (Type, Expression),
    /// Function location.
    pub location: Location,
}

impl Function {
    /// [Type] the function.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     constant::Constant, calculus::Calculus, function::Function, location::Location,
    ///     expression::Expression, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    /// let user_types_context = HashMap::new();
    ///
    /// let mut function = Function {
    ///     id: String::from("test"),
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     calculi: vec![
    ///         (
    ///             String::from("x"),
    ///             Calculus {
    ///                 id: String::from("x"),
    ///                 element_type: Type::Integer,
    ///                 expression: Expression::Call {
    ///                     id: String::from("i"),
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ],
    ///     returned: (
    ///         Type::Integer,
    ///         Expression::Call {
    ///             id: String::from("x"),
    ///             typing: None,
    ///             location: Location::default(),
    ///         }
    ///     ),
    ///     location: Location::default(),
    /// };
    ///
    /// function.typing(&user_types_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), Error> {
        let Function {
            inputs,
            calculi,
            returned: (returned_type, returned_expression),
            location,
            ..
        } = self;

        // create elements context: global_context + inputs
        let mut elements_context = global_context::generate();
        inputs
            .iter()
            .map(|(name, input_type)| {
                elements_context.insert_unique(
                    name.clone(),
                    input_type.clone(),
                    location.clone(),
                    errors,
                )
            })
            .collect::<Vec<Result<(), Error>>>()
            .into_iter()
            .collect::<Result<(), Error>>()?;

        // type all calculi
        calculi
            .iter_mut()
            .map(|(_, calculus)| {
                calculus.typing(&elements_context, user_types_context, errors)?;
                elements_context.insert_unique(
                    calculus.id.clone(),
                    calculus.element_type.clone(),
                    calculus.location.clone(),
                    errors,
                )
            })
            .collect::<Vec<Result<(), Error>>>()
            .into_iter()
            .collect::<Result<(), Error>>()?;

        // type returned expression
        returned_expression.typing(&elements_context, user_types_context, errors)?;

        // check returned type
        returned_expression
            .get_type()
            .unwrap()
            .eq_check(returned_type, location.clone(), errors)
    }
}

#[cfg(test)]
mod typing {
    use std::collections::HashMap;

    use crate::ast::{
        calculus::Calculus, expression::Expression, function::Function, location::Location,
        type_system::Type,
    };

    #[test]
    fn should_type_well_defined_function() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();

        let mut function = Function {
            id: String::from("test"),
            inputs: vec![(String::from("i"), Type::Integer)],
            calculi: vec![(
                String::from("x"),
                Calculus {
                    id: String::from("x"),
                    element_type: Type::Integer,
                    expression: Expression::Call {
                        id: String::from("i"),
                        typing: None,
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
            )],
            returned: (
                Type::Integer,
                Expression::Call {
                    id: String::from("x"),
                    typing: None,
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };

        let control = Function {
            id: String::from("test"),
            inputs: vec![(String::from("i"), Type::Integer)],
            calculi: vec![(
                String::from("x"),
                Calculus {
                    id: String::from("x"),
                    element_type: Type::Integer,
                    expression: Expression::Call {
                        id: String::from("i"),
                        typing: Some(Type::Integer),
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
            )],
            returned: (
                Type::Integer,
                Expression::Call {
                    id: String::from("x"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };

        function.typing(&user_types_context, &mut errors).unwrap();

        assert_eq!(function, control)
    }

    #[test]
    fn should_raise_error_for_incompatible_type_in_function() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();

        let mut function = Function {
            id: String::from("test"),
            inputs: vec![(String::from("i"), Type::Float)],
            calculi: vec![(
                String::from("x"),
                Calculus {
                    id: String::from("x"),
                    element_type: Type::Float,
                    expression: Expression::Call {
                        id: String::from("i"),
                        typing: None,
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
            )],
            returned: (
                Type::Integer,
                Expression::Call {
                    id: String::from("x"),
                    typing: None,
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };

        let error = function
            .typing(&user_types_context, &mut errors)
            .unwrap_err();

        assert_eq!(errors, vec![error])
    }
}
