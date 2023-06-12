use std::collections::HashMap;

use crate::ast::{
    component::Component, function::Function, global_context, location::Location, node::Node,
    type_system::Type, user_defined_type::UserDefinedType,
};

use crate::common::context::Context;
use crate::error::Error;

#[derive(Debug, PartialEq)]
/// Enumerates the different kinds of files in LanGRust.
pub enum File {
    /// A LanGRust [File::Module] is composed of functions
    /// nodes and types defined by the user.
    Module {
        /// Module types.
        user_defined_types: Vec<UserDefinedType>,
        /// Module functions.
        functions: Vec<Function>,
        /// Module nodes. They are functional requirements.
        nodes: Vec<Node>,
        /// Module location.
        location: Location,
    },
    /// A LanGRust [File::Program] is composed of functions
    /// nodes and types defined by the user and a component.
    Program {
        /// Program types.
        user_defined_types: Vec<UserDefinedType>,
        /// Program functions.
        functions: Vec<Function>,
        /// Program nodes. They are functional requirements.
        nodes: Vec<Node>,
        /// Program component. It represents the system.
        component: Component,
        /// Program location.
        location: Location,
    },
}
impl File {
    /// Get types definitions from a LanGRust file.
    pub fn get_types(self) -> Vec<UserDefinedType> {
        match self {
            File::Module {
                user_defined_types,
                functions: _,
                nodes: _,
                location: _,
            } => user_defined_types,
            File::Program {
                user_defined_types,
                functions: _,
                nodes: _,
                component: _,
                location: _,
            } => user_defined_types,
        }
    }
    /// Get functions from a LanGRust file.
    pub fn get_functions(self) -> Vec<Function> {
        match self {
            File::Module {
                user_defined_types: _,
                functions,
                nodes: _,
                location: _,
            } => functions,
            File::Program {
                user_defined_types: _,
                functions,
                nodes: _,
                component: _,
                location: _,
            } => functions,
        }
    }
    /// Get nodes from a LanGRust file.
    pub fn get_nodes(self) -> Vec<Node> {
        match self {
            File::Module {
                user_defined_types: _,
                functions: _,
                nodes,
                location: _,
            } => nodes,
            File::Program {
                user_defined_types: _,
                functions: _,
                nodes,
                component: _,
                location: _,
            } => nodes,
        }
    }
    /// Get types, functions and nodes from a LanGRust file.
    pub fn get_types_functions_nodes(self) -> (Vec<UserDefinedType>, Vec<Function>, Vec<Node>) {
        match self {
            File::Module {
                user_defined_types,
                functions,
                nodes,
                location: _,
            } => (user_defined_types, functions, nodes),
            File::Program {
                user_defined_types,
                functions,
                nodes,
                component: _,
                location: _,
            } => (user_defined_types, functions, nodes),
        }
    }
    /// Get the location of a LanGRust file.
    pub fn get_location(self) -> Location {
        match self {
            File::Module {
                user_defined_types: _,
                functions: _,
                nodes: _,
                location,
            } => location,
            File::Program {
                user_defined_types: _,
                functions: _,
                nodes: _,
                component: _,
                location,
            } => location,
        }
    }
    /// Add a type definition to a LanGRust file functions.
    pub fn push_type(&mut self, user_defined_type: UserDefinedType) {
        match self {
            File::Module {
                ref mut user_defined_types,
                functions: _,
                nodes: _,
                location: _,
            } => user_defined_types.push(user_defined_type),
            File::Program {
                ref mut user_defined_types,
                functions: _,
                nodes: _,
                component: _,
                location: _,
            } => user_defined_types.push(user_defined_type),
        }
    }
    /// Add a function to a LanGRust file functions.
    pub fn push_function(&mut self, function: Function) {
        match self {
            File::Module {
                user_defined_types: _,
                ref mut functions,
                nodes: _,
                location: _,
            } => functions.push(function),
            File::Program {
                user_defined_types: _,
                ref mut functions,
                nodes: _,
                component: _,
                location: _,
            } => functions.push(function),
        }
    }
    /// Add a node to a LanGRust file nodes.
    pub fn push_node(&mut self, node: Node) {
        match self {
            File::Module {
                user_defined_types: _,
                functions: _,
                ref mut nodes,
                location: _,
            } => nodes.push(node),
            File::Program {
                user_defined_types: _,
                functions: _,
                ref mut nodes,
                component: _,
                location: _,
            } => nodes.push(node),
        }
    }
    /// Change the location of a LanGRust file.
    pub fn set_location(&mut self, new_location: Location) {
        match self {
            File::Module {
                user_defined_types: _,
                functions: _,
                nodes: _,
                ref mut location,
            } => *location = new_location,
            File::Program {
                user_defined_types: _,
                functions: _,
                nodes: _,
                component: _,
                ref mut location,
            } => *location = new_location,
        }
    }

    /// [Type] the entire file.
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::{
    ///     statement::Statement, function::Function, location::Location,
    ///     expression::Expression, type_system::Type, equation::Equation, node::Node, file::File,
    ///     scope::Scope, stream_expression::StreamExpression,
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
    /// let function = Function {
    ///     id: String::from("test"),
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     statements: vec![
    ///         (
    ///             String::from("x"),
    ///             Statement {
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
    /// let mut file = File::Module {
    ///     user_defined_types: vec![],
    ///     functions: vec![function],
    ///     nodes: vec![node],
    ///     location: Location::default(),
    /// };
    ///
    /// file.typing(&mut errors).unwrap();
    /// ```
    pub fn typing(&mut self, errors: &mut Vec<Error>) -> Result<(), ()> {
        match self {
            File::Module {
                user_defined_types,
                functions,
                nodes,
                ..
            } => {
                // create user_types_context
                let mut user_types_context = HashMap::new();
                user_defined_types
                    .iter()
                    .map(|user_type| match user_type {
                        UserDefinedType::Structure { id, location, .. }
                        | UserDefinedType::Enumeration { id, location, .. }
                        | UserDefinedType::Array { id, location, .. } => user_types_context
                            .insert_unique(id.clone(), user_type.clone(), location.clone(), errors),
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // determine types in user_defined_types
                user_defined_types
                    .iter_mut()
                    .map(|user_type| user_type.determine_types(&user_types_context, errors))
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // recreate a user_types_context with determined types
                let mut user_types_context = HashMap::new();
                user_defined_types
                    .iter()
                    .map(|user_type| match user_type {
                        UserDefinedType::Structure { id, location, .. }
                        | UserDefinedType::Enumeration { id, location, .. }
                        | UserDefinedType::Array { id, location, .. } => user_types_context
                            .insert_unique(id.clone(), user_type.clone(), location.clone(), errors),
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // determine types in nodes
                nodes
                    .iter_mut()
                    .map(|node| node.determine_types(&user_types_context, errors))
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // determine types in functions
                functions
                    .iter_mut()
                    .map(|function| function.determine_types(&user_types_context, errors))
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // create nodes_context
                let mut nodes_context = HashMap::new();
                nodes
                    .iter()
                    .map(|node| {
                        let node_description = node.into_node_description(errors)?;
                        nodes_context.insert_unique(
                            node.id.clone(),
                            node_description,
                            node.location.clone(),
                            errors,
                        )
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // generate global_context
                let mut global_context = global_context::generate();

                // add functions to global_context
                functions
                    .iter()
                    .map(
                        |Function {
                             id,
                             inputs,
                             returned: (returned_type, _),
                             location,
                             ..
                         }| {
                            let function_type = inputs.iter().rev().fold(
                                returned_type.clone(),
                                |current_type, (_, input_type)| {
                                    Type::Abstract(
                                        Box::new(input_type.clone()),
                                        Box::new(current_type),
                                    )
                                },
                            );
                            global_context.insert_unique(
                                id.clone(),
                                function_type,
                                location.clone(),
                                errors,
                            )
                        },
                    )
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // typing nodes
                nodes
                    .iter_mut()
                    .map(|node| {
                        node.typing(&nodes_context, &global_context, &user_types_context, errors)
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // typing functions
                functions
                    .iter_mut()
                    .map(|function| function.typing(&global_context, &user_types_context, errors))
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()
            }
            File::Program {
                user_defined_types,
                functions,
                nodes,
                component,
                ..
            } => {
                // create user_types_context
                let mut user_types_context = HashMap::new();
                user_defined_types
                    .iter()
                    .map(|user_type| match user_type {
                        UserDefinedType::Structure { id, location, .. }
                        | UserDefinedType::Enumeration { id, location, .. }
                        | UserDefinedType::Array { id, location, .. } => user_types_context
                            .insert_unique(id.clone(), user_type.clone(), location.clone(), errors),
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // determine types in user_defined_types
                user_defined_types
                    .iter_mut()
                    .map(|user_type| user_type.determine_types(&user_types_context, errors))
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // recreate a user_types_context with determined types
                let mut user_types_context = HashMap::new();
                user_defined_types
                    .iter()
                    .map(|user_type| match user_type {
                        UserDefinedType::Structure { id, location, .. }
                        | UserDefinedType::Enumeration { id, location, .. }
                        | UserDefinedType::Array { id, location, .. } => user_types_context
                            .insert_unique(id.clone(), user_type.clone(), location.clone(), errors),
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // determine types in nodes
                nodes
                    .iter_mut()
                    .map(|node| node.determine_types(&user_types_context, errors))
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // determine types in component
                component.determine_types(&user_types_context, errors)?;

                // determine types in functions
                functions
                    .iter_mut()
                    .map(|function| function.determine_types(&user_types_context, errors))
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // create nodes_context
                let mut nodes_context = HashMap::new();
                nodes
                    .iter()
                    .map(|node| {
                        let node_description = node.into_node_description(errors)?;
                        nodes_context.insert_unique(
                            node.id.clone(),
                            node_description,
                            node.location.clone(),
                            errors,
                        )
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // generate global_context
                let mut global_context = global_context::generate();

                // add functions to global_context
                functions
                    .iter()
                    .map(
                        |Function {
                             id,
                             inputs,
                             returned: (returned_type, _),
                             location,
                             ..
                         }| {
                            let function_type = inputs.iter().rev().fold(
                                returned_type.clone(),
                                |current_type, (_, input_type)| {
                                    Type::Abstract(
                                        Box::new(input_type.clone()),
                                        Box::new(current_type),
                                    )
                                },
                            );
                            global_context.insert_unique(
                                id.clone(),
                                function_type,
                                location.clone(),
                                errors,
                            )
                        },
                    )
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // typing nodes
                nodes
                    .iter_mut()
                    .map(|node| {
                        node.typing(&nodes_context, &global_context, &user_types_context, errors)
                    })
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()?;

                // typing component
                component.typing(&nodes_context, &global_context, &user_types_context, errors)?;

                // typing functions
                functions
                    .iter_mut()
                    .map(|function| function.typing(&global_context, &user_types_context, errors))
                    .collect::<Vec<Result<(), ()>>>()
                    .into_iter()
                    .collect::<Result<(), ()>>()
            }
        }
    }
}

#[cfg(test)]
mod typing {
    use crate::ast::{
        component::Component, equation::Equation, expression::Expression, file::File,
        function::Function, location::Location, node::Node, scope::Scope, statement::Statement,
        stream_expression::StreamExpression, type_system::Type,
    };

    #[test]
    fn should_type_well_defined_module() {
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

        let function = Function {
            id: String::from("test"),
            inputs: vec![(String::from("i"), Type::Integer)],
            statements: vec![(
                String::from("x"),
                Statement {
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

        let mut file = File::Module {
            user_defined_types: vec![],
            functions: vec![function],
            nodes: vec![node],
            location: Location::default(),
        };

        let expected_node = Node {
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

        let expected_function = Function {
            id: String::from("test"),
            inputs: vec![(String::from("i"), Type::Integer)],
            statements: vec![(
                String::from("x"),
                Statement {
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

        let control = File::Module {
            user_defined_types: vec![],
            functions: vec![expected_function],
            nodes: vec![expected_node],
            location: Location::default(),
        };

        file.typing(&mut errors).unwrap();

        assert_eq!(file, control);
    }

    #[test]
    fn should_type_well_defined_program() {
        let mut errors = vec![];

        let component = Component {
            id: String::from("program_component"),
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("test"),
                                typing: None,
                                location: Location::default(),
                            },
                            inputs: vec![StreamExpression::SignalCall {
                                id: String::from("x"),
                                typing: None,
                                location: Location::default(),
                            }],
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
                        expression: StreamExpression::NodeApplication {
                            node: String::from("test"),
                            inputs: vec![StreamExpression::SignalCall {
                                id: String::from("i"),
                                typing: None,
                                location: Location::default(),
                            }],
                            signal: String::from("o"),
                            typing: None,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        };

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

        let function = Function {
            id: String::from("test"),
            inputs: vec![(String::from("i"), Type::Integer)],
            statements: vec![(
                String::from("x"),
                Statement {
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

        let mut file = File::Program {
            user_defined_types: vec![],
            functions: vec![function],
            nodes: vec![node],
            component,
            location: Location::default(),
        };

        let expected_component = Component {
            id: String::from("program_component"),
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("test"),
                                typing: Some(Type::Abstract(
                                    Box::new(Type::Integer),
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![StreamExpression::SignalCall {
                                id: String::from("x"),
                                typing: Some(Type::Integer),
                                location: Location::default(),
                            }],
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
                        expression: StreamExpression::NodeApplication {
                            node: String::from("test"),
                            inputs: vec![StreamExpression::SignalCall {
                                id: String::from("i"),
                                typing: Some(Type::Integer),
                                location: Location::default(),
                            }],
                            signal: String::from("o"),
                            typing: Some(Type::Integer),
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ],
            location: Location::default(),
        };

        let expected_node = Node {
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

        let expected_function = Function {
            id: String::from("test"),
            inputs: vec![(String::from("i"), Type::Integer)],
            statements: vec![(
                String::from("x"),
                Statement {
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

        let control = File::Program {
            user_defined_types: vec![],
            functions: vec![expected_function],
            nodes: vec![expected_node],
            component: expected_component,
            location: Location::default(),
        };

        file.typing(&mut errors).unwrap();

        assert_eq!(file, control);
    }
}
