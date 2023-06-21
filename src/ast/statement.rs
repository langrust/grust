use std::collections::HashMap;

use crate::ast::{expression::Expression, user_defined_type::UserDefinedType};
use crate::common::{location::Location, type_system::Type};
use crate::error::Error;
use crate::ir::statement::Statement as IRStatement;

#[derive(Debug, PartialEq, Clone)]
/// LanGRust statement AST.
pub struct Statement {
    /// Identifier of the new element.
    pub id: String,
    /// Element type.
    pub element_type: Type,
    /// The expression defining the element.
    pub expression: Expression,
    /// Statement location.
    pub location: Location,
}

impl Statement {
    /// [Type] the statement.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::{
    ///     expression::Expression, statement::Statement,
    /// };
    /// use grustine::common::{
    ///     constant::Constant, location::Location, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    /// let global_context = HashMap::new();
    /// let elements_context = HashMap::new();
    /// let user_types_context = HashMap::new();
    ///
    /// let mut expression = Expression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: None,
    ///     location: Location::default(),
    /// };
    /// let mut statement = Statement {
    ///     id: String::from("x"),
    ///     element_type: Type::Integer,
    ///     expression: expression,
    ///     location: Location::default(),
    /// };
    ///
    /// statement.typing(&global_context, &elements_context, &user_types_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        global_context: &HashMap<String, Type>,
        elements_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Statement {
            element_type,
            expression,
            location,
            ..
        } = self;

        expression.typing(global_context, elements_context, user_types_context, errors)?;

        let expression_type = expression.get_type().unwrap();

        expression_type.eq_check(element_type, location.clone(), errors)
    }

    /// Determine the type of the statement if undefined
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::{
    ///     expression::Expression, statement::Statement, user_defined_type::UserDefinedType,
    /// };
    /// use grustine::common::{
    ///     constant::Constant, location::Location, type_system::Type,
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
    ///     }
    /// );
    ///
    /// let mut statement = Statement {
    ///     id: String::from("o"),
    ///     element_type: Type::NotDefinedYet(String::from("Point")),
    ///     expression: Expression::Structure {
    ///         name: String::from("Point"),
    ///         fields: vec![
    ///             (
    ///                 String::from("x"),
    ///                 Expression::Constant {
    ///                     constant: Constant::Integer(1),
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///             ),
    ///             (
    ///                 String::from("y"),
    ///                 Expression::Constant {
    ///                     constant: Constant::Integer(2),
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///             ),
    ///         ],
    ///         typing: None,
    ///         location: Location::default(),
    ///     },
    ///     location: Location::default(),
    /// };
    ///
    /// let control = Statement {
    ///     id: String::from("o"),
    ///     element_type: Type::Structure(String::from("Point")),
    ///     expression: Expression::Structure {
    ///         name: String::from("Point"),
    ///         fields: vec![
    ///             (
    ///                 String::from("x"),
    ///                 Expression::Constant {
    ///                     constant: Constant::Integer(1),
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///             ),
    ///             (
    ///                 String::from("y"),
    ///                 Expression::Constant {
    ///                     constant: Constant::Integer(2),
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///             ),
    ///         ],
    ///         typing: None,
    ///         location: Location::default(),
    ///     },
    ///     location: Location::default(),
    /// };
    ///
    /// calculus
    ///     .resolve_undefined_types(&user_types_context, &mut errors)
    ///     .unwrap();
    ///
    /// assert_eq!(statement, control);
    /// ```
    pub fn resolve_undefined_types(
        &mut self,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Statement {
            element_type,
            location,
            ..
        } = self;
        element_type.resolve_undefined(location.clone(), user_types_context, errors)
    }

    /// Transform AST statements into IR statements.
    pub fn into_ir(self) -> IRStatement {
        let Statement {
            id,
            element_type,
            expression,
            location,
        } = self;

        IRStatement {
            id,
            element_type,
            expression: expression.into_ir(),
            location,
        }
    }
}

#[cfg(test)]
mod typing {
    use std::collections::HashMap;

    use crate::ast::{expression::Expression, statement::Statement};
    use crate::common::{constant::Constant, location::Location, type_system::Type};

    #[test]
    fn should_type_well_defined_equation() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let elements_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut statement = Statement {
            id: String::from("x"),
            element_type: Type::Integer,
            expression: Expression::Constant {
                constant: Constant::Integer(0),
                typing: None,
                location: Location::default(),
            },
            location: Location::default(),
        };

        let control = Statement {
            id: String::from("x"),
            element_type: Type::Integer,
            expression: Expression::Constant {
                constant: Constant::Integer(0),
                typing: Some(Type::Integer),
                location: Location::default(),
            },
            location: Location::default(),
        };

        statement
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(statement, control)
    }

    #[test]
    fn should_raise_error_for_incompatible_type_in_equation() {
        let mut errors = vec![];
        let global_context = HashMap::new();
        let elements_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut statement = Statement {
            id: String::from("x"),
            element_type: Type::Float,
            expression: Expression::Constant {
                constant: Constant::Integer(0),
                typing: None,
                location: Location::default(),
            },
            location: Location::default(),
        };

        statement
            .typing(
                &global_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}

#[cfg(test)]
mod determine_types {
    use std::collections::HashMap;

    use crate::ast::{
        expression::Expression, statement::Statement, user_defined_type::UserDefinedType,
    };
    use crate::common::{constant::Constant, location::Location, type_system::Type};

    #[test]
    fn should_determine_the_type_of_equation_when_in_types_context() {
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

        let mut statement = Statement {
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
        };

        let control = Statement {
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
        };

        calculus
            .resolve_undefined_types(&user_types_context, &mut errors)
            .unwrap();

        assert_eq!(statement, control);
    }

    #[test]
    fn should_raise_error_when_undefined_type_not_in_types_context() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();

        let mut statement = Statement {
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
        };

        calculus
            .resolve_undefined_types(&user_types_context, &mut errors)
            .unwrap_err();
    }
}
