use std::collections::HashMap;

use crate::ast::{
    node_description::NodeDescription, stream_expression::StreamExpression, typedef::Typedef,
};
use crate::common::{location::Location, scope::Scope, type_system::Type};
use crate::error::Error;

#[derive(Debug, PartialEq, Clone)]
/// LanGRust equation AST.
pub struct Equation {
    /// Signal's scope.
    pub scope: Scope,
    /// Identifier of the signal.
    pub id: String,
    /// Signal type.
    pub signal_type: Type,
    /// The stream expression defining the signal.
    pub expression: StreamExpression,
    /// Equation location.
    pub location: Location,
}

impl Equation {
    /// [Type] the equation.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::{equation::Equation, stream_expression::StreamExpression};
    /// use grustine::common::{
    ///     constant::Constant, location::Location, scope::Scope, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    /// let nodes_context = HashMap::new();
    /// let signals_context = HashMap::new();
    /// let elements_context = HashMap::new();
    /// let user_types_context = HashMap::new();
    ///
    /// let mut stream_expression = StreamExpression::Constant {
    ///     constant: Constant::Integer(0),
    ///     typing: None,
    ///     location: Location::default(),
    /// };
    /// let mut equation = Equation {
    ///     scope: Scope::Local,
    ///     id: String::from("s"),
    ///     signal_type: Type::Integer,
    ///     expression: stream_expression,
    ///     location: Location::default(),
    /// };
    ///
    /// equation.typing(&nodes_context, &signals_context, &elements_context, &user_types_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        signals_context: &HashMap<String, Type>,
        elements_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Equation {
            signal_type,
            expression,
            location,
            ..
        } = self;

        expression.typing(
            nodes_context,
            signals_context,
            elements_context,
            user_types_context,
            errors,
        )?;

        let expression_type = expression.get_type().unwrap();

        expression_type.eq_check(signal_type, location.clone(), errors)
    }

    /// Determine the type of the equation if undefined
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ast::{
    ///     equation::Equation, stream_expression::StreamExpression, typedef::Typedef,
    /// };
    /// use grustine::common::{
    ///     constant::Constant, location::Location, scope::Scope, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    /// let mut user_types_context = HashMap::new();
    /// user_types_context.insert(
    ///     String::from("Point"),
    ///     Typedef::Structure {
    ///         id: String::from("Point"),
    ///         fields: vec![
    ///             (String::from("x"), Type::Integer),
    ///             (String::from("y"), Type::Integer),
    ///         ],
    ///         location: Location::default(),
    ///     }
    /// );
    ///
    /// let mut equation = Equation {
    ///     scope: Scope::Output,
    ///     id: String::from("o"),
    ///     signal_type: Type::NotDefinedYet(String::from("Point")),
    ///     expression: StreamExpression::Structure {
    ///         name: String::from("Point"),
    ///         fields: vec![
    ///             (
    ///                 String::from("x"),
    ///                 StreamExpression::Constant {
    ///                     constant: Constant::Integer(1),
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///             ),
    ///             (
    ///                 String::from("y"),
    ///                 StreamExpression::Constant {
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
    /// let control = Equation {
    ///     scope: Scope::Output,
    ///     id: String::from("o"),
    ///     signal_type: Type::Structure(String::from("Point")),
    ///     expression: StreamExpression::Structure {
    ///         name: String::from("Point"),
    ///         fields: vec![
    ///             (
    ///                 String::from("x"),
    ///                 StreamExpression::Constant {
    ///                     constant: Constant::Integer(1),
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///             ),
    ///             (
    ///                 String::from("y"),
    ///                 StreamExpression::Constant {
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
    /// equation
    ///     .resolve_undefined_types(&user_types_context, &mut errors)
    ///     .unwrap();
    ///
    /// assert_eq!(equation, control);
    /// ```
    pub fn resolve_undefined_types(
        &mut self,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Equation {
            signal_type,
            location,
            ..
        } = self;
        signal_type.resolve_undefined(location.clone(), user_types_context, errors)
    }
}

#[cfg(test)]
mod typing {
    use std::collections::HashMap;

    use crate::ast::{equation::Equation, stream_expression::StreamExpression};
    use crate::common::{constant::Constant, location::Location, scope::Scope, type_system::Type};

    #[test]
    fn should_type_well_defined_equation() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let signals_context = HashMap::new();
        let elements_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut equation = Equation {
            scope: Scope::Local,
            id: String::from("s"),
            signal_type: Type::Integer,
            expression: StreamExpression::Constant {
                constant: Constant::Integer(0),
                typing: None,
                location: Location::default(),
            },
            location: Location::default(),
        };

        let control = Equation {
            scope: Scope::Local,
            id: String::from("s"),
            signal_type: Type::Integer,
            expression: StreamExpression::Constant {
                constant: Constant::Integer(0),
                typing: Some(Type::Integer),
                location: Location::default(),
            },
            location: Location::default(),
        };

        equation
            .typing(
                &nodes_context,
                &signals_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(equation, control)
    }

    #[test]
    fn should_raise_error_for_incompatible_type_in_equation() {
        let mut errors = vec![];
        let nodes_context = HashMap::new();
        let signals_context = HashMap::new();
        let elements_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut equation = Equation {
            scope: Scope::Local,
            id: String::from("s"),
            signal_type: Type::Float,
            expression: StreamExpression::Constant {
                constant: Constant::Integer(0),
                typing: None,
                location: Location::default(),
            },
            location: Location::default(),
        };

        equation
            .typing(
                &nodes_context,
                &signals_context,
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

    use crate::ast::{equation::Equation, stream_expression::StreamExpression, typedef::Typedef};
    use crate::common::{constant::Constant, location::Location, scope::Scope, type_system::Type};

    #[test]
    fn should_determine_the_type_of_equation_when_in_types_context() {
        let mut errors = vec![];
        let mut user_types_context = HashMap::new();
        user_types_context.insert(
            String::from("Point"),
            Typedef::Structure {
                id: String::from("Point"),
                fields: vec![
                    (String::from("x"), Type::Integer),
                    (String::from("y"), Type::Integer),
                ],
                location: Location::default(),
            },
        );

        let mut equation = Equation {
            scope: Scope::Output,
            id: String::from("o"),
            signal_type: Type::NotDefinedYet(String::from("Point")),
            expression: StreamExpression::Structure {
                name: String::from("Point"),
                fields: vec![
                    (
                        String::from("x"),
                        StreamExpression::Constant {
                            constant: Constant::Integer(1),
                            typing: None,
                            location: Location::default(),
                        },
                    ),
                    (
                        String::from("y"),
                        StreamExpression::Constant {
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

        let control = Equation {
            scope: Scope::Output,
            id: String::from("o"),
            signal_type: Type::Structure(String::from("Point")),
            expression: StreamExpression::Structure {
                name: String::from("Point"),
                fields: vec![
                    (
                        String::from("x"),
                        StreamExpression::Constant {
                            constant: Constant::Integer(1),
                            typing: None,
                            location: Location::default(),
                        },
                    ),
                    (
                        String::from("y"),
                        StreamExpression::Constant {
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

        equation
            .resolve_undefined_types(&user_types_context, &mut errors)
            .unwrap();

        assert_eq!(equation, control);
    }

    #[test]
    fn should_raise_error_when_undefined_type_not_in_types_context() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();

        let mut equation = Equation {
            scope: Scope::Output,
            id: String::from("o"),
            signal_type: Type::NotDefinedYet(String::from("Point")),
            expression: StreamExpression::Structure {
                name: String::from("Point"),
                fields: vec![
                    (
                        String::from("x"),
                        StreamExpression::Constant {
                            constant: Constant::Integer(1),
                            typing: None,
                            location: Location::default(),
                        },
                    ),
                    (
                        String::from("y"),
                        StreamExpression::Constant {
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

        equation
            .resolve_undefined_types(&user_types_context, &mut errors)
            .unwrap_err();
    }
}
