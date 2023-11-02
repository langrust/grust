use std::collections::HashMap;

use crate::ast::{function::Function, global_context, node::Node, typedef::Typedef};
use crate::common::{context::Context, location::Location, r#type::Type};
use crate::error::{Error, TerminationError};

#[derive(Debug, PartialEq, serde::Serialize)]
/// A LanGRust [File] is composed of functions nodes,
/// types defined by the user and an optional component.
pub struct File {
    /// Program types.
    pub typedefs: Vec<Typedef>,
    /// Program functions.
    pub functions: Vec<Function>,
    /// Program nodes. They are functional requirements.
    pub nodes: Vec<Node>,
    /// Program component. It represents the system.
    pub component: Option<Node>,
    /// Program location.
    pub location: Location,
}

impl File {
    /// Get types definitions from a LanGRust file.
    pub fn get_types(self) -> Vec<Typedef> {
        self.typedefs
    }
    /// Get functions from a LanGRust file.
    pub fn get_functions(self) -> Vec<Function> {
        self.functions
    }
    /// Get nodes from a LanGRust file.
    pub fn get_nodes(self) -> Vec<Node> {
        self.nodes
    }
    /// Get types, functions and nodes from a LanGRust file.
    pub fn get_types_functions_nodes(self) -> (Vec<Typedef>, Vec<Function>, Vec<Node>) {
        (self.typedefs, self.functions, self.nodes)
    }
    /// Get the location of a LanGRust file.
    pub fn get_location(self) -> Location {
        self.location
    }
    /// Add a type definition to a LanGRust file functions.
    pub fn push_type(&mut self, typedef: Typedef) {
        self.typedefs.push(typedef)
    }
    /// Add a function to a LanGRust file functions.
    pub fn push_function(&mut self, function: Function) {
        self.functions.push(function)
    }
    /// Add a node to a LanGRust file nodes.
    pub fn push_node(&mut self, node: Node) {
        self.nodes.push(node)
    }
    /// Change the location of a LanGRust file.
    pub fn set_location(&mut self, new_location: Location) {
        self.location = new_location
    }

    /// [Type] the entire file.
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::{
    ///     equation::Equation, expression::Expression, function::Function,
    ///     file::File, node::Node, statement::Statement, stream_expression::StreamExpression,
    /// };
    /// use grustine::common::{location::Location, scope::Scope, r#type::Type};
    ///
    /// let mut errors = vec![];
    ///
    /// let node = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
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
    /// let mut file = File {
    ///     typedefs: vec![],
    ///     functions: vec![function],
    ///     nodes: vec![node],
    ///     component: None,
    ///     location: Location::default(),
    /// };
    ///
    /// file.typing(&mut errors).unwrap();
    /// ```
    pub fn typing(&mut self, errors: &mut Vec<Error>) -> Result<(), TerminationError> {
        let File {
            typedefs,
            functions,
            nodes,
            component,
            ..
        } = self;

        // create user_types_context
        let mut user_types_context = HashMap::new();
        typedefs
            .iter()
            .map(|user_type| match user_type {
                Typedef::Structure { id, location, .. }
                | Typedef::Enumeration { id, location, .. }
                | Typedef::Array { id, location, .. } => user_types_context.insert_unique(
                    id.clone(),
                    user_type.clone(),
                    location.clone(),
                    errors,
                ),
            })
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // resolve undefined types in typedefs
        typedefs
            .iter_mut()
            .map(|user_type| user_type.resolve_undefined_types(&user_types_context, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // recreate a user_types_context with resolved types
        let mut user_types_context = HashMap::new();
        typedefs
            .iter()
            .map(|user_type| match user_type {
                Typedef::Structure { id, location, .. }
                | Typedef::Enumeration { id, location, .. }
                | Typedef::Array { id, location, .. } => user_types_context.insert_unique(
                    id.clone(),
                    user_type.clone(),
                    location.clone(),
                    errors,
                ),
            })
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // resolve undefined types in nodes
        nodes
            .iter_mut()
            .map(|node| node.resolve_undefined_types(&user_types_context, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // resolve undefined types in component
        component.as_mut().map_or(Ok(()), |component| {
            component.resolve_undefined_types(&user_types_context, errors)
        })?;

        // resolve undefined types in functions
        functions
            .iter_mut()
            .map(|function| function.resolve_undefined_types(&user_types_context, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

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
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // add component to context
        component.as_ref().map_or(Ok(()), |component| {
            let node_description = component.into_node_description(errors)?;
            nodes_context.insert_unique(
                component.id.clone(),
                node_description,
                component.location.clone(),
                errors,
            )
        })?;

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
                    let input_types = inputs
                        .iter()
                        .map(|(_, input_type)| input_type.clone())
                        .collect();
                    let function_type =
                        Type::Abstract(input_types, Box::new(returned_type.clone()));
                    global_context.insert_unique(
                        id.clone(),
                        function_type,
                        location.clone(),
                        errors,
                    )
                },
            )
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // typing nodes
        nodes
            .iter_mut()
            .map(|node| node.typing(&nodes_context, &global_context, &user_types_context, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // typing component
        component.as_mut().map_or(Ok(()), |component| {
            component.typing(&nodes_context, &global_context, &user_types_context, errors)
        })?;

        // typing functions
        functions
            .iter_mut()
            .map(|function| function.typing(&global_context, &user_types_context, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()
    }
}

#[cfg(test)]
mod typing {
    use crate::ast::{
        equation::Equation, expression::Expression, file::File, function::Function, node::Node,
        statement::Statement, stream_expression::StreamExpression,
    };
    use crate::common::{location::Location, r#type::Type, scope::Scope};

    #[test]
    fn should_type_well_defined_module() {
        let mut errors = vec![];

        let node = Node {
            id: String::from("test"),
            is_component: false,
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

        let mut file = File {
            typedefs: vec![],
            functions: vec![function],
            nodes: vec![node],
            component: None,
            location: Location::default(),
        };

        let expected_node = Node {
            id: String::from("test"),
            is_component: false,
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

        let control = File {
            typedefs: vec![],
            functions: vec![expected_function],
            nodes: vec![expected_node],
            component: None,
            location: Location::default(),
        };

        file.typing(&mut errors).unwrap();

        assert_eq!(file, control);
    }

    #[test]
    fn should_type_well_defined_program() {
        let mut errors = vec![];

        let component = Node {
            id: String::from("program_component"),
            is_component: true,
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
            is_component: false,
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

        let mut file = File {
            typedefs: vec![],
            functions: vec![function],
            nodes: vec![node],
            component: Some(component),
            location: Location::default(),
        };

        let expected_component = Node {
            id: String::from("program_component"),
            is_component: true,
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
                                    vec![Type::Integer],
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
            is_component: false,
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

        let control = File {
            typedefs: vec![],
            functions: vec![expected_function],
            nodes: vec![expected_node],
            component: Some(expected_component),
            location: Location::default(),
        };

        file.typing(&mut errors).unwrap();

        assert_eq!(file, control);
    }
}
