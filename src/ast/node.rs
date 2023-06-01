use std::collections::HashMap;

use crate::ast::{
    equation::Equation, location::Location, node_description::NodeDescription, scope::Scope,
    type_system::Type, user_defined_type::UserDefinedType,
};
use crate::common::context::Context;
use crate::error::Error;

#[derive(Debug, PartialEq)]
/// LanGRust node AST.
pub struct Node {
    /// Node identifier.
    pub id: String,
    /// Node's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Node's equations.
    pub equations: Vec<(String, Equation)>,
    /// Node location.
    pub location: Location,
}

impl Node {
    /// [Type] the node.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     constant::Constant, equation::Equation, location::Location, node::Node,
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
    /// let global_context = HashMap::new();
    /// let user_types_context = HashMap::new();
    ///
    /// let mut node = Node {
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
    /// node.typing(&nodes_context, &global_context, &user_types_context, &mut errors).unwrap();
    /// ```
    pub fn typing(
        &mut self,
        nodes_context: &HashMap<String, NodeDescription>,
        global_context: &HashMap<String, Type>,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Node {
            id,
            equations,
            location,
            ..
        } = self;

        // get the description of the node
        let NodeDescription {
            inputs,
            outputs,
            locals,
        } = nodes_context.get_node_or_error(id, location.clone(), errors)?;

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
                    global_context,
                    user_types_context,
                    errors,
                )
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()
    }

    /// Create a [NodeDescription] from a [Node]
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     equation::Equation, location::Location, node::Node, node_description::NodeDescription,
    ///     scope::Scope, stream_expression::StreamExpression, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    ///
    /// let node = Node {
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
    /// let node_description = node.into_node_description(&mut errors).unwrap();
    ///
    /// assert_eq!(node_description, control);
    /// ```
    pub fn into_node_description(&self, errors: &mut Vec<Error>) -> Result<NodeDescription, ()> {
        let Node {
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

    /// Determine all undefined types in node
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     constant::Constant, node::Node,
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
    /// let mut node = Node {
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
    /// let control = Node {
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
    /// node
    ///     .determine_types(&user_types_context, &mut errors)
    ///     .unwrap();
    ///
    /// assert_eq!(node, control);
    /// ```
    pub fn determine_types(
        &mut self,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Node {
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

    /// Create a [NodeDescription] from a [Node]
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     equation::Equation, location::Location, node::Node, node_description::NodeDescription,
    ///     scope::Scope, stream_expression::StreamExpression, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    ///
    /// let node = Node {
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
    /// let node_description = node.into_node_description(&mut errors).unwrap();
    ///
    /// assert_eq!(node_description, control);
    /// ```
    pub fn into_node_description(&self, errors: &mut Vec<Error>) -> Result<NodeDescription, ()> {
        let Node {
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

    /// Determine all undefined types in node
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     constant::Constant, node::Node,
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
    /// let mut node = Node {
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
    /// let control = Node {
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
    /// node
    ///     .determine_types(&user_types_context, &mut errors)
    ///     .unwrap();
    ///
    /// assert_eq!(node, control);
    /// ```
    pub fn determine_types(
        &mut self,
        user_types_context: &HashMap<String, UserDefinedType>,
        errors: &mut Vec<Error>,
    ) -> Result<(), ()> {
        let Node {
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

    /// Create a [NodeDescription] from a [Node]
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    /// use grustine::ast::{
    ///     equation::Equation, location::Location, node::Node, node_description::NodeDescription,
    ///     scope::Scope, stream_expression::StreamExpression, type_system::Type,
    /// };
    ///
    /// let mut errors = vec![];
    ///
    /// let node = Node {
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
    /// let node_description = node.into_node_description(&mut errors).unwrap();
    ///
    /// assert_eq!(node_description, control);
    /// ```
    pub fn into_node_description(&self, errors: &mut Vec<Error>) -> Result<NodeDescription, Error> {
        let Node {
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
            .collect::<Vec<Result<(), Error>>>()
            .into_iter()
            .collect::<Result<(), Error>>()?;

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
            .collect::<Vec<Result<(), Error>>>()
            .into_iter()
            .collect::<Result<(), Error>>()?;

        Ok(NodeDescription {
            inputs: inputs.clone(),
            outputs,
            locals,
        })
    }
}

#[cfg(test)]
mod typing {
    use std::collections::HashMap;

    use crate::ast::{
        constant::Constant, equation::Equation, location::Location, node::Node,
        node_description::NodeDescription, scope::Scope, stream_expression::StreamExpression,
        type_system::Type,
    };

    #[test]
    fn should_type_well_defined_node() {
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
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut node = Node {
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

        let control = Node {
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

        node.typing(
            &nodes_context,
            &global_context,
            &user_types_context,
            &mut errors,
        )
        .unwrap();

        assert_eq!(node, control)
    }

    #[test]
    fn should_raise_error_for_incompatible_type_in_node() {
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
        let global_context = HashMap::new();
        let user_types_context = HashMap::new();

        let mut node = Node {
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

        node.typing(
            &nodes_context,
            &global_context,
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
        equation::Equation, location::Location, node::Node, node_description::NodeDescription,
        scope::Scope, stream_expression::StreamExpression, type_system::Type,
    };
    #[test]
    fn should_return_a_node_description_from_a_node_with_no_duplicates() {
        let mut errors = vec![];

        let node = Node {
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

        let node_description = node.into_node_description(&mut errors).unwrap();

        assert_eq!(node_description, control);
    }
}

#[cfg(test)]
mod determine_types {
    use crate::ast::{
        constant::Constant, equation::Equation, location::Location, node::Node, scope::Scope,
        stream_expression::StreamExpression, type_system::Type, user_defined_type::UserDefinedType,
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

        let mut node = Node {
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

        let control = Node {
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

        node.determine_types(&user_types_context, &mut errors)
            .unwrap();

        assert_eq!(node, control);
    }

    #[test]
    fn should_raise_error_when_undefined_types_not_in_context() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();

        let mut node = Node {
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

        node.determine_types(&user_types_context, &mut errors)
            .unwrap_err();
    }
}

#[cfg(test)]
mod into_node_description {
    use std::collections::HashMap;

    use crate::ast::{
        equation::Equation, location::Location, node::Node, node_description::NodeDescription,
        scope::Scope, stream_expression::StreamExpression, type_system::Type,
    };
    #[test]
    fn should_return_a_node_description_from_a_node_with_no_duplicates() {
        let mut errors = vec![];

        let node = Node {
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

        let node_description = node.into_node_description(&mut errors).unwrap();

        assert_eq!(node_description, control);
    }
}

#[cfg(test)]
mod determine_types {
    use crate::ast::{
        constant::Constant, equation::Equation, location::Location, node::Node, scope::Scope,
        stream_expression::StreamExpression, type_system::Type, user_defined_type::UserDefinedType,
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

        let mut node = Node {
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

        let control = Node {
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

        node.determine_types(&user_types_context, &mut errors)
            .unwrap();

        assert_eq!(node, control);
    }

    #[test]
    fn should_raise_error_when_undefined_types_not_in_context() {
        let mut errors = vec![];
        let user_types_context = HashMap::new();

        let mut node = Node {
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

        node.determine_types(&user_types_context, &mut errors)
            .unwrap_err();
    }
}

#[cfg(test)]
mod into_node_description {
    use std::collections::HashMap;

    use crate::ast::{
        equation::Equation, location::Location, node::Node, node_description::NodeDescription,
        scope::Scope, stream_expression::StreamExpression, type_system::Type,
    };
    #[test]
    fn should_return_a_node_description_from_a_node_with_no_duplicates() {
        let mut errors = vec![];

        let node = Node {
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

        let node_description = node.into_node_description(&mut errors).unwrap();

        assert_eq!(node_description, control);
    }
}
