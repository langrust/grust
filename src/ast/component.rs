use std::collections::HashMap;

use crate::ast::{
    equation::Equation, location::Location, node_description::NodeDescription, scope::Scope,
    type_system::Type, user_defined_type::UserDefinedType,
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
    ) -> Result<(), ()> {
        // get the description of the component
        let NodeDescription {
            inputs,
            outputs,
            locals,
        } = self.into_node_description(errors)?;

        // match the component
        let Component {
            equations,
            location,
            ..
        } = self;

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
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;
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
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()
    }

    /// Create a [NodeDescription] from a [Component]
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     equation::Equation, location::Location, component::Component, node_description::NodeDescription,
    ///     scope::Scope, stream_expression::StreamExpression, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    ///
    /// let component = Component {
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
    /// let control = NodeDescription {
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     outputs: HashMap::from([(String::from("o"), Type::Integer)]),
    ///     locals: HashMap::from([(String::from("x"), Type::Integer)]),
    /// };
    ///
    /// let node_description = component.into_node_description(&mut errors).unwrap();
    ///
    /// assert_eq!(node_description, control);
    /// ```
    pub fn into_node_description(&self, errors: &mut Vec<Error>) -> Result<NodeDescription, ()> {
        let Component {
            inputs,
            equations,
            location,
            ..
        } = self;

        // differenciate output form local signals
        let mut outputs = HashMap::new();
        let mut locals = HashMap::new();

        // create signals context: inputs + outputs + locals
        // and check that no signal is duplicated
        let mut signals_context = HashMap::new();

        // add inputs in signals context
        inputs
            .iter()
            .map(|(id, signal_type)| {
                signals_context.insert_unique(
                    id.clone(),
                    signal_type.clone(),
                    location.clone(),
                    errors,
                )
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // add signals defined by equations in contexts
        equations
            .iter()
            .map(
                |(
                    _,
                    Equation {
                        scope,
                        id,
                        signal_type,
                        location,
                        ..
                    },
                )| {
                    // differenciate output form local signals
                    match scope {
                        Scope::Output => outputs.insert(id.clone(), signal_type.clone()),
                        Scope::Local => locals.insert(id.clone(), signal_type.clone()),
                        _ => unreachable!(),
                    };
                    // check that no signal is duplicated
                    signals_context.insert_unique(
                        id.clone(),
                        signal_type.clone(),
                        location.clone(),
                        errors,
                    )
                },
            )
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        Ok(NodeDescription {
            inputs: inputs.clone(),
            outputs,
            locals,
        })
    }

    /// Determine all undefined types in component
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     constant::Constant, component::Component,
    ///     equation::Equation, stream_expression::StreamExpression, scope::Scope,
    ///     location::Location, type_system::Type, user_defined_type::UserDefinedType,
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
    /// let mut component = Component {
    ///     id: String::from("test"),
    ///     inputs: vec![],
    ///     equations: vec![
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::NotDefinedYet(String::from("Point")),
    ///                 expression: StreamExpression::Structure {
    ///                     name: String::from("Point"),
    ///                     fields: vec![
    ///                         (
    ///                             String::from("x"),
    ///                             StreamExpression::Constant {
    ///                                 constant: Constant::Integer(1),
    ///                                 typing: None,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ),
    ///                         (
    ///                             String::from("y"),
    ///                             StreamExpression::Constant {
    ///                                 constant: Constant::Integer(2),
    ///                                 typing: None,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ),
    ///                     ],
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///     ],
    ///     location: Location::default(),
    /// };
    ///
    /// let control = Component {
    ///     id: String::from("test"),
    ///     inputs: vec![],
    ///     equations: vec![
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Structure(String::from("Point")),
    ///                 expression: StreamExpression::Structure {
    ///                     name: String::from("Point"),
    ///                     fields: vec![
    ///                         (
    ///                             String::from("x"),
    ///                             StreamExpression::Constant {
    ///                                 constant: Constant::Integer(1),
    ///                                 typing: None,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ),
    ///                         (
    ///                             String::from("y"),
    ///                             StreamExpression::Constant {
    ///                                 constant: Constant::Integer(2),
    ///                                 typing: None,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ),
    ///                     ],
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///     ],
    ///     location: Location::default(),
    /// };
    ///
    /// component
    ///     .determine_types(&user_types_context, &mut errors)
    ///     .unwrap();
    ///
    /// assert_eq!(component, control);
    /// ```
    pub fn determine_types(
        &mut self,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Component {
            inputs,
            equations,
            location,
            ..
        } = self;

        // determine inputs types
        inputs
            .iter_mut()
            .map(|(_, input_type)| {
                input_type.determine(location.clone(), user_types_context, errors)
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // determine equations types
        equations
            .iter_mut()
            .map(|(_, equation)| equation.determine_types(user_types_context, errors))
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()
    }

    /// Create a [NodeDescription] from a [Component]
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     equation::Equation, location::Location, component::Component, node_description::NodeDescription,
    ///     scope::Scope, stream_expression::StreamExpression, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    ///
    /// let component = Component {
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
    /// let control = NodeDescription {
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     outputs: HashMap::from([(String::from("o"), Type::Integer)]),
    ///     locals: HashMap::from([(String::from("x"), Type::Integer)]),
    /// };
    ///
    /// let node_description = component.into_node_description(&mut errors).unwrap();
    ///
    /// assert_eq!(node_description, control);
    /// ```
    pub fn into_node_description(&self, errors: &mut Vec<Error>) -> Result<NodeDescription, ()> {
        let Component {
            inputs,
            equations,
            location,
            ..
        } = self;

        // differenciate output form local signals
        let mut outputs = HashMap::new();
        let mut locals = HashMap::new();

        // create signals context: inputs + outputs + locals
        // and check that no signal is duplicated
        let mut signals_context = HashMap::new();

        // add inputs in signals context
        inputs
            .iter()
            .map(|(id, signal_type)| {
                signals_context.insert_unique(
                    id.clone(),
                    signal_type.clone(),
                    location.clone(),
                    errors,
                )
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // add signals defined by equations in contexts
        equations
            .iter()
            .map(
                |(
                    _,
                    Equation {
                        scope,
                        id,
                        signal_type,
                        location,
                        ..
                    },
                )| {
                    // differenciate output form local signals
                    match scope {
                        Scope::Output => outputs.insert(id.clone(), signal_type.clone()),
                        Scope::Local => locals.insert(id.clone(), signal_type.clone()),
                        _ => unreachable!(),
                    };
                    // check that no signal is duplicated
                    signals_context.insert_unique(
                        id.clone(),
                        signal_type.clone(),
                        location.clone(),
                        errors,
                    )
                },
            )
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        Ok(NodeDescription {
            inputs: inputs.clone(),
            outputs,
            locals,
        })
    }

    /// Determine all undefined types in component
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     constant::Constant, component::Component,
    ///     equation::Equation, stream_expression::StreamExpression, scope::Scope,
    ///     location::Location, type_system::Type, user_defined_type::UserDefinedType,
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
    /// let mut component = Component {
    ///     id: String::from("test"),
    ///     inputs: vec![],
    ///     equations: vec![
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::NotDefinedYet(String::from("Point")),
    ///                 expression: StreamExpression::Structure {
    ///                     name: String::from("Point"),
    ///                     fields: vec![
    ///                         (
    ///                             String::from("x"),
    ///                             StreamExpression::Constant {
    ///                                 constant: Constant::Integer(1),
    ///                                 typing: None,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ),
    ///                         (
    ///                             String::from("y"),
    ///                             StreamExpression::Constant {
    ///                                 constant: Constant::Integer(2),
    ///                                 typing: None,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ),
    ///                     ],
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///     ],
    ///     location: Location::default(),
    /// };
    ///
    /// let control = Component {
    ///     id: String::from("test"),
    ///     inputs: vec![],
    ///     equations: vec![
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Structure(String::from("Point")),
    ///                 expression: StreamExpression::Structure {
    ///                     name: String::from("Point"),
    ///                     fields: vec![
    ///                         (
    ///                             String::from("x"),
    ///                             StreamExpression::Constant {
    ///                                 constant: Constant::Integer(1),
    ///                                 typing: None,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ),
    ///                         (
    ///                             String::from("y"),
    ///                             StreamExpression::Constant {
    ///                                 constant: Constant::Integer(2),
    ///                                 typing: None,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ),
    ///                     ],
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///     ],
    ///     location: Location::default(),
    /// };
    ///
    /// component
    ///     .determine_types(&user_types_context, &mut errors)
    ///     .unwrap();
    ///
    /// assert_eq!(component, control);
    /// ```
    pub fn determine_types(
        &mut self,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Component {
            inputs,
            equations,
            location,
            ..
        } = self;

        // determine inputs types
        inputs
            .iter_mut()
            .map(|(_, input_type)| {
                input_type.determine(location.clone(), user_types_context, errors)
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // determine equations types
        equations
            .iter_mut()
            .map(|(_, equation)| equation.determine_types(user_types_context, errors))
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()
    }

    /// Create a [NodeDescription] from a [Component]
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     equation::Equation, location::Location, component::Component, node_description::NodeDescription,
    ///     scope::Scope, stream_expression::StreamExpression, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    ///
    /// let component = Component {
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
    /// let control = NodeDescription {
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     outputs: HashMap::from([(String::from("o"), Type::Integer)]),
    ///     locals: HashMap::from([(String::from("x"), Type::Integer)]),
    /// };
    ///
    /// let node_description = component.into_node_description(&mut errors).unwrap();
    ///
    /// assert_eq!(node_description, control);
    /// ```
    pub fn into_node_description(&self, errors: &mut Vec<Error>) -> Result<NodeDescription, ()> {
        let Component {
            inputs,
            equations,
            location,
            ..
        } = self;

        // differenciate output form local signals
        let mut outputs = HashMap::new();
        let mut locals = HashMap::new();

        // create signals context: inputs + outputs + locals
        // and check that no signal is duplicated
        let mut signals_context = HashMap::new();

        // add inputs in signals context
        inputs
            .iter()
            .map(|(id, signal_type)| {
                signals_context.insert_unique(
                    id.clone(),
                    signal_type.clone(),
                    location.clone(),
                    errors,
                )
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // add signals defined by equations in contexts
        equations
            .iter()
            .map(
                |(
                    _,
                    Equation {
                        scope,
                        id,
                        signal_type,
                        location,
                        ..
                    },
                )| {
                    // differenciate output form local signals
                    match scope {
                        Scope::Output => outputs.insert(id.clone(), signal_type.clone()),
                        Scope::Local => locals.insert(id.clone(), signal_type.clone()),
                        _ => unreachable!(),
                    };
                    // check that no signal is duplicated
                    signals_context.insert_unique(
                        id.clone(),
                        signal_type.clone(),
                        location.clone(),
                        errors,
                    )
                },
            )
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        Ok(NodeDescription {
            inputs: inputs.clone(),
            outputs,
            locals,
        })
    }

    /// Determine all undefined types in component
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     constant::Constant, component::Component,
    ///     equation::Equation, stream_expression::StreamExpression, scope::Scope,
    ///     location::Location, type_system::Type, user_defined_type::UserDefinedType,
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
    /// let mut component = Component {
    ///     id: String::from("test"),
    ///     inputs: vec![],
    ///     equations: vec![
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::NotDefinedYet(String::from("Point")),
    ///                 expression: StreamExpression::Structure {
    ///                     name: String::from("Point"),
    ///                     fields: vec![
    ///                         (
    ///                             String::from("x"),
    ///                             StreamExpression::Constant {
    ///                                 constant: Constant::Integer(1),
    ///                                 typing: None,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ),
    ///                         (
    ///                             String::from("y"),
    ///                             StreamExpression::Constant {
    ///                                 constant: Constant::Integer(2),
    ///                                 typing: None,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ),
    ///                     ],
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///     ],
    ///     location: Location::default(),
    /// };
    ///
    /// let control = Component {
    ///     id: String::from("test"),
    ///     inputs: vec![],
    ///     equations: vec![
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Structure(String::from("Point")),
    ///                 expression: StreamExpression::Structure {
    ///                     name: String::from("Point"),
    ///                     fields: vec![
    ///                         (
    ///                             String::from("x"),
    ///                             StreamExpression::Constant {
    ///                                 constant: Constant::Integer(1),
    ///                                 typing: None,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ),
    ///                         (
    ///                             String::from("y"),
    ///                             StreamExpression::Constant {
    ///                                 constant: Constant::Integer(2),
    ///                                 typing: None,
    ///                                 location: Location::default(),
    ///                             },
    ///                         ),
    ///                     ],
    ///                     typing: None,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         ),
    ///     ],
    ///     location: Location::default(),
    /// };
    ///
    /// component
    ///     .determine_types(&user_types_context, &mut errors)
    ///     .unwrap();
    ///
    /// assert_eq!(component, control);
    /// ```
    pub fn determine_types(
        &mut self,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Component {
            inputs,
            equations,
            location,
            ..
        } = self;

        // determine inputs types
        inputs
            .iter_mut()
            .map(|(_, input_type)| {
                input_type.determine(location.clone(), user_types_context, errors)
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // determine equations types
        equations
            .iter_mut()
            .map(|(_, equation)| equation.determine_types(user_types_context, errors))
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()
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

        component
            .typing(
                &nodes_context,
                &elements_context,
                &user_types_context,
                &mut errors,
            )
            .unwrap_err();
    }
}

#[cfg(test)]
mod into_node_description {
    use std::collections::HashMap;

    use crate::ast::{
        component::Component, equation::Equation, location::Location,
        node_description::NodeDescription, scope::Scope, stream_expression::StreamExpression,
        type_system::Type,
    };
    #[test]
    fn should_return_a_node_description_from_a_component_with_no_duplicates() {
        let mut errors = vec![];

        let component = Component {
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

        let control = NodeDescription {
            inputs: vec![(String::from("i"), Type::Integer)],
            outputs: HashMap::from([(String::from("o"), Type::Integer)]),
            locals: HashMap::from([(String::from("x"), Type::Integer)]),
        };

        let node_description = component.into_node_description(&mut errors).unwrap();

        assert_eq!(node_description, control);
    }
}

#[cfg(test)]
mod determine_types {
    use crate::ast::{
        component::Component, constant::Constant, equation::Equation, location::Location,
        scope::Scope, stream_expression::StreamExpression, type_system::Type,
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

        let mut component = Component {
            id: String::from("test"),
            inputs: vec![],
            equations: vec![(
                String::from("o"),
                Equation {
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
                },
            )],
            location: Location::default(),
        };

        let control = Component {
            id: String::from("test"),
            inputs: vec![],
            equations: vec![(
                String::from("o"),
                Equation {
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
                },
            )],
            location: Location::default(),
        };

        component
            .determine_types(&user_types_context, &mut errors)
            .unwrap();

        assert_eq!(component, control);
    }

    #[test]
    fn should_raise_error_when_undefined_types_not_in_context() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();

        let mut component = Component {
            id: String::from("test"),
            inputs: vec![],
            equations: vec![(
                String::from("o"),
                Equation {
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
                },
            )],
            location: Location::default(),
        };

        component
            .determine_types(&user_types_context, &mut errors)
            .unwrap_err();
    }
}

#[cfg(test)]
mod into_node_description {
    use std::collections::HashMap;

    use crate::ast::{
        component::Component, equation::Equation, location::Location,
        node_description::NodeDescription, scope::Scope, stream_expression::StreamExpression,
        type_system::Type,
    };
    #[test]
    fn should_return_a_node_description_from_a_component_with_no_duplicates() {
        let mut errors = vec![];

        let component = Component {
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

        let control = NodeDescription {
            inputs: vec![(String::from("i"), Type::Integer)],
            outputs: HashMap::from([(String::from("o"), Type::Integer)]),
            locals: HashMap::from([(String::from("x"), Type::Integer)]),
        };

        let node_description = component.into_node_description(&mut errors).unwrap();

        assert_eq!(node_description, control);
    }
}

#[cfg(test)]
mod determine_types {
    use crate::ast::{
        component::Component, constant::Constant, equation::Equation, location::Location,
        scope::Scope, stream_expression::StreamExpression, type_system::Type,
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

        let mut component = Component {
            id: String::from("test"),
            inputs: vec![],
            equations: vec![(
                String::from("o"),
                Equation {
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
                },
            )],
            location: Location::default(),
        };

        let control = Component {
            id: String::from("test"),
            inputs: vec![],
            equations: vec![(
                String::from("o"),
                Equation {
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
                },
            )],
            location: Location::default(),
        };

        component
            .determine_types(&user_types_context, &mut errors)
            .unwrap();

        assert_eq!(component, control);
    }

    #[test]
    fn should_raise_error_when_undefined_types_not_in_context() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();

        let mut component = Component {
            id: String::from("test"),
            inputs: vec![],
            equations: vec![(
                String::from("o"),
                Equation {
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
                },
            )],
            location: Location::default(),
        };

        component
            .determine_types(&user_types_context, &mut errors)
            .unwrap_err();
    }
}

#[cfg(test)]
mod into_node_description {
    use std::collections::HashMap;

    use crate::ast::{
        component::Component, equation::Equation, location::Location,
        node_description::NodeDescription, scope::Scope, stream_expression::StreamExpression,
        type_system::Type,
    };
    #[test]
    fn should_return_a_node_description_from_a_component_with_no_duplicates() {
        let mut errors = vec![];

        let component = Component {
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

        let control = NodeDescription {
            inputs: vec![(String::from("i"), Type::Integer)],
            outputs: HashMap::from([(String::from("o"), Type::Integer)]),
            locals: HashMap::from([(String::from("x"), Type::Integer)]),
        };

        let node_description = component.into_node_description(&mut errors).unwrap();

        assert_eq!(node_description, control);
    }
}

#[cfg(test)]
mod determine_types {
    use crate::ast::{
        component::Component, constant::Constant, equation::Equation, location::Location,
        scope::Scope, stream_expression::StreamExpression, type_system::Type,
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

        let mut component = Component {
            id: String::from("test"),
            inputs: vec![],
            equations: vec![(
                String::from("o"),
                Equation {
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
                },
            )],
            location: Location::default(),
        };

        let control = Component {
            id: String::from("test"),
            inputs: vec![],
            equations: vec![(
                String::from("o"),
                Equation {
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
                },
            )],
            location: Location::default(),
        };

        component
            .determine_types(&user_types_context, &mut errors)
            .unwrap();

        assert_eq!(component, control);
    }

    #[test]
    fn should_raise_error_when_undefined_types_not_in_context() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();

        let mut component = Component {
            id: String::from("test"),
            inputs: vec![],
            equations: vec![(
                String::from("o"),
                Equation {
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
                },
            )],
            location: Location::default(),
        };

        component
            .determine_types(&user_types_context, &mut errors)
            .unwrap_err();
    }
}
