use std::collections::HashMap;

use crate::ast::{
    location::Location, node_description::NodeDescription, scope::Scope,
    stream_expression::StreamExpression, type_system::Type, user_defined_type::UserDefinedType,
};
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
    /// Add a [Type] to the equation.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     constant::Constant, equation::Equation, location::Location,
    ///     scope::Scope, stream_expression::StreamExpression, type_system::Type,
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
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), Error> {
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
}

#[cfg(test)]
mod typing {
    use std::collections::HashMap;

    use crate::ast::{
        constant::Constant, equation::Equation, location::Location, scope::Scope,
        stream_expression::StreamExpression, type_system::Type,
    };

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
}
