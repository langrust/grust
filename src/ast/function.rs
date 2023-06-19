use std::collections::HashMap;

use crate::ast::{expression::Expression, statement::Statement};
use crate::common::{
    context::Context, location::Location, type_system::Type, user_defined_type::UserDefinedType,
};
use crate::error::Error;
use crate::ir::function::Function as IRFunction;

#[derive(Debug, PartialEq)]
/// LanGRust function AST.
pub struct Function {
    /// Function identifier.
    pub id: String,
    /// Function's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Function's statements.
    pub statements: Vec<Statement>,
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
    ///
    /// use grustine::ast::{
    ///     expression::Expression, function::Function, statement::Statement,
    /// };
    /// use grustine::common::{
    ///     constant::Constant, location::Location, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    /// let global_context = HashMap::new();
    /// let user_types_context = HashMap::new();
    ///
    /// let mut function = Function {
    ///     id: String::from("test"),
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     statements: vec![
    ///         Statement {
    ///             id: String::from("x"),
    ///             element_type: Type::Integer,
    ///             expression: Expression::Call {
    ///                 id: String::from("i"),
    ///                 typing: None,
    ///                 location: Location::default(),
    ///             },
    ///             location: Location::default(),
    ///         }
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
    /// function.typing(&global_context, &user_types_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Function {
            inputs,
            statements,
            returned: (returned_type, returned_expression),
            location,
            ..
        } = self;

        // create elements context: global_context + inputs
        let mut elements_context = global_context.clone();
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
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // type all statements
        statements
            .iter_mut()
            .map(|statement| {
                statement.typing(
                    global_context,
                    &elements_context,
                    user_types_context,
                    errors,
                )?;
                elements_context.insert_unique(
                    statement.id.clone(),
                    statement.element_type.clone(),
                    statement.location.clone(),
                    errors,
                )
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // type returned expression
        returned_expression.typing(
            global_context,
            &elements_context,
            user_types_context,
            errors,
        )?;

        // check returned type
        returned_expression
            .get_type()
            .unwrap()
            .eq_check(returned_type, location.clone(), errors)
    }

    /// Determine all undefined types in function
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::{
    ///     expression::Expression, function::Function, statement::Statement,
    /// };
    /// use grustine::common::{
    ///     constant::Constant, location::Location, type_system::Type, user_defined_type::UserDefinedType,
    /// };
    ///
    /// let mut errors = vec![];
    /// let mut user_types_context = HashMap::new();
    /// user_types_context.insert(
    ///     String::from("Point"),
    ///     UserDefinedType::Structure {
    ///         id: String::from("Point"),
    ///         fields: vec![
    ///             (String::from("x"), Type::Integer),
    ///             (String::from("y"), Type::Integer),
    ///         ],
    ///         location: Location::default(),
    ///     },
    /// );
    ///
    /// let mut function = Function {
    ///     id: String::from("test"),
    ///     inputs: vec![],
    ///     statements: vec![
    ///         Statement {
    ///             id: String::from("o"),
    ///             element_type: Type::NotDefinedYet(String::from("Point")),
    ///             expression: Expression::Structure {
    ///                 name: String::from("Point"),
    ///                 fields: vec![
    ///                     (
    ///                         String::from("x"),
    ///                         Expression::Constant {
    ///                             constant: Constant::Integer(1),
    ///                             typing: None,
    ///                             location: Location::default(),
    ///                         },
    ///                     ),
    ///                     (
    ///                         String::from("y"),
    ///                         Expression::Constant {
    ///                             constant: Constant::Integer(2),
    ///                             typing: None,
    ///                             location: Location::default(),
    ///                         },
    ///                     ),
    ///                 ],
    ///                 typing: None,
    ///                 location: Location::default(),
    ///             },
    ///             location: Location::default(),
    ///         },
    ///     ],
    ///     returned: (Type::NotDefinedYet(String::from("Point")), Expression::Call {
    ///         id: String::from("o"),
    ///         typing: None,
    ///         location: Location::default(),
    ///     }),
    ///     location: Location::default(),
    /// };
    ///
    /// let control = Function {
    ///     id: String::from("test"),
    ///     inputs: vec![],
    ///     statements: vec![
    ///         Statement {
    ///             id: String::from("o"),
    ///             element_type: Type::Structure(String::from("Point")),
    ///             expression: Expression::Structure {
    ///                 name: String::from("Point"),
    ///                 fields: vec![
    ///                     (
    ///                         String::from("x"),
    ///                         Expression::Constant {
    ///                             constant: Constant::Integer(1),
    ///                             typing: None,
    ///                             location: Location::default(),
    ///                         },
    ///                     ),
    ///                     (
    ///                         String::from("y"),
    ///                         Expression::Constant {
    ///                             constant: Constant::Integer(2),
    ///                             typing: None,
    ///                             location: Location::default(),
    ///                         },
    ///                     ),
    ///                 ],
    ///                 typing: None,
    ///                 location: Location::default(),
    ///             },
    ///             location: Location::default(),
    ///         },
    ///     ],
    ///     returned: (Type::Structure(String::from("Point")), Expression::Call {
    ///         id: String::from("o"),
    ///         typing: None,
    ///         location: Location::default(),
    ///     }),
    ///     location: Location::default(),
    /// };
    ///
    /// function.resolve_undefined_types(&user_types_context, &mut errors)
    ///     .unwrap();
    ///
    /// assert_eq!(function, control);
    /// ```
    pub fn resolve_undefined_types(
        &mut self,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Function {
            inputs,
            statements,
            returned: (returned_type, _),
            location,
            ..
        } = self;

        // determine inputs types
        inputs
            .iter_mut()
            .map(|(_, input_type)| {
                input_type.resolve_undefined(location.clone(), user_types_context, errors)
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // determine statements types
        statements
            .iter_mut()
            .map(|statement| statement.resolve_undefined_types(user_types_context, errors))
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // determine returned type
        returned_type.resolve_undefined(location.clone(), user_types_context, errors)
    }

    /// Transform AST functions into IR function.
    pub fn into_ir(self) -> IRFunction {
        let Function {
            id,
            inputs,
            statements,
            returned: (returned_type, returned_expression),
            location,
        } = self;

        IRFunction {
            id,
            inputs,
            statements: statements
                .into_iter()
                .map(|statement| statement.into_ir())
                .collect(),
            returned: (returned_type, returned_expression.into_ir()),
            location,
        }
    }
}

#[cfg(test)]
mod typing {
    use std::collections::HashMap;

    use crate::ast::{expression::Expression, function::Function, statement::Statement};
    use crate::common::{location::Location, type_system::Type};

    #[test]
    fn should_type_well_defined_function() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut function = Function {
            id: String::from("test"),
            inputs: vec![(String::from("i"), Type::Integer)],
            statements: vec![Statement {
                id: String::from("x"),
                element_type: Type::Integer,
                expression: Expression::Call {
                    id: String::from("i"),
                    typing: None,
                    location: Location::default(),
                },
                location: Location::default(),
            }],
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
            statements: vec![Statement {
                id: String::from("x"),
                element_type: Type::Integer,
                expression: Expression::Call {
                    id: String::from("i"),
                    typing: Some(Type::Integer),
                    location: Location::default(),
                },
                location: Location::default(),
            }],
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

        function
            .typing(&global_context, &user_types_context, &mut errors)
            .unwrap();

        assert_eq!(function, control)
    }

    #[test]
    fn should_raise_error_for_incompatible_type_in_function() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut function = Function {
            id: String::from("test"),
            inputs: vec![(String::from("i"), Type::Float)],
            statements: vec![Statement {
                id: String::from("x"),
                element_type: Type::Float,
                expression: Expression::Call {
                    id: String::from("i"),
                    typing: None,
                    location: Location::default(),
                },
                location: Location::default(),
            }],
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

        function
            .typing(&global_context, &user_types_context, &mut errors)
            .unwrap_err();
    }
}

#[cfg(test)]
mod resolve_undefined_types {
    use crate::ast::{expression::Expression, function::Function, statement::Statement};
    use crate::common::{
        constant::Constant, location::Location, type_system::Type,
        user_defined_type::UserDefinedType,
    };
    use std::collections::HashMap;

    #[test]
    fn should_determine_undefined_types_when_in_context() {
        let mut errors = vec![];
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

        let mut function = Function {
            id: String::from("test"),
            inputs: vec![],
            statements: vec![Statement {
                id: String::from("o"),
                element_type: Type::NotDefinedYet(String::from("Point")),
                expression: Expression::Structure {
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
                },
                location: Location::default(),
            }],
            returned: (
                Type::NotDefinedYet(String::from("Point")),
                Expression::Call {
                    id: String::from("o"),
                    typing: None,
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };

        let control = Function {
            id: String::from("test"),
            inputs: vec![],
            statements: vec![Statement {
                id: String::from("o"),
                element_type: Type::Structure(String::from("Point")),
                expression: Expression::Structure {
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
                },
                location: Location::default(),
            }],
            returned: (
                Type::Structure(String::from("Point")),
                Expression::Call {
                    id: String::from("o"),
                    typing: None,
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };

        function
            .resolve_undefined_types(&user_types_context, &mut errors)
            .unwrap();

        assert_eq!(function, control);
    }

    #[test]
    fn should_raise_error_when_undefined_types_not_in_context() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();

        let mut function = Function {
            id: String::from("test"),
            inputs: vec![],
            statements: vec![Statement {
                id: String::from("o"),
                element_type: Type::NotDefinedYet(String::from("Point")),
                expression: Expression::Structure {
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
                },
                location: Location::default(),
            }],
            returned: (
                Type::NotDefinedYet(String::from("Point")),
                Expression::Call {
                    id: String::from("o"),
                    typing: None,
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };

        function
            .resolve_undefined_types(&user_types_context, &mut errors)
            .unwrap_err();
    }
}
