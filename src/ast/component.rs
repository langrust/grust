use std::collections::HashMap;

use crate::ast::{
    equation::Equation, location::Location, node_description::NodeDescription, type_system::Type,
    user_defined_type::UserDefinedType,
};
use crate::common::context::Context;
use crate::error::Error;

#[derive(Debug, PartialEq)]
/// LanGRust component AST.
pub struct Component {
    /// Component identifier.
    pub id: String,
    /// Component's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Component's equations.
    pub equations: Vec<(String, Equation)>,
    /// Component location.
    pub location: Location,
}

impl Component {
    /// [Type] the component.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     constant::Constant, equation::Equation, location::Location, component::Component,
    ///     node_description::NodeDescription, scope::Scope,
    ///     stream_expression::StreamExpression, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    /// let mut nodes_context = HashMap::new();
    /// nodes_context.insert(
    ///     String::from("test"),
    ///     NodeDescription {
    ///         inputs: vec![(String::from("i"), Type::Integer)],
    ///         outputs: HashMap::from([(String::from("o"), Type::Integer)]),
    ///         locals: HashMap::from([(String::from("x"), Type::Integer)]),
    ///     }
    /// );
    /// let elements_context = HashMap::new();
    /// let user_types_context = HashMap::new();
    ///
    /// let mut component = Component {
    ///     id: String::from("test"),
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     equations: vec![
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///         (
    ///             String::from("x"),
    ///             Equation {
    ///                 scope: Scope::Local,
    ///                 id: String::from("x"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("i"),
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ],
    ///     location: Location::default(),
    /// };
    ///
    /// component.typing(&nodes_context, &elements_context, &user_types_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        elements_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), Error> {
        let Component {
            id,
            equations,
            location,
            ..
        } = self;

        // get the description of the component
        let NodeDescription {
            inputs,
            outputs,
            locals,
        } = nodes_context.get_node_or_error(id.clone(), location.clone(), errors)?;

        // create signals context: inputs + outputs + locals
        let mut signals_context = HashMap::new();
        inputs
            .iter()
            .map(|(name, input_type)| {
                signals_context.insert_unique(
                    name.clone(),
                    input_type.clone(),
                    location.clone(),
                    errors,
                )
            })
            .collect::<Vec<Result<(), Error>>>()
            .into_iter()
            .collect::<Result<(), Error>>()?;
        signals_context.combine_unique(outputs.clone(), location.clone(), errors)?;
        signals_context.combine_unique(locals.clone(), location.clone(), errors)?;

        // type all equations
        equations
            .iter_mut()
            .map(|(_, equation)| {
                equation.typing(
                    nodes_context,
                    &signals_context,
                    elements_context,
                    user_types_context,
                    errors,
                )
            })
            .collect::<Vec<Result<(), Error>>>()
            .into_iter()
            .collect::<Result<(), Error>>()
    }
}

#[cfg(test)]
mod typing {
    use std::collections::HashMap;

    use crate::ast::{
        component::Component, constant::Constant, equation::Equation, location::Location,
        node_description::NodeDescription, scope::Scope, stream_expression::StreamExpression,
        type_system::Type,
    };

    #[test]
    fn should_type_well_defined_component() {
        let mut errors = vec![];
        let mut nodes_context = HashMap::new();
        nodes_context.insert(
            String::from("test"),
            NodeDescription {
                inputs: vec![(String::from("i"), Type::Integer)],
                outputs: HashMap::from([(String::from("o"), Type::Integer)]),
                locals: HashMap::from([(String::from("x"), Type::Integer)]),
            },
        );
        let elements_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut component = Component {
            id: String::from("test"),
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("x"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("x"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i"),
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        };

        let control = Component {
            id: String::from("test"),
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Some(Type::Integer),
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("x"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("x"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i"),
                            typing: Some(Type::Integer),
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        };

        component
            .typing(
                &nodes_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap();

        assert_eq!(component, control)
    }

    #[test]
    fn should_raise_error_for_incompatible_type_in_component() {
        let mut errors = vec![];
        let mut nodes_context = HashMap::new();
        nodes_context.insert(
            String::from("test"),
            NodeDescription {
                inputs: vec![(String::from("i"), Type::Integer)],
                outputs: HashMap::from([(String::from("o"), Type::Integer)]),
                locals: HashMap::from([(String::from("x"), Type::Integer)]),
            },
        );
        let elements_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut component = Component {
            id: String::from("test"),
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::Constant {
                            constant: Constant::Float(0.1),
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("x"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("x"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("i"),
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        };

        let error = component
            .typing(
                &nodes_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();

        assert_eq!(errors, vec![error])
    }
}
