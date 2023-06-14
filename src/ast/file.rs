use std::collections::HashMap;

use crate::ast::{
    function::Function, global_context, location::Location, node::Node, type_system::Type,
    user_defined_type::UserDefinedType,
};

use crate::common::{color::Color, context::Context, graph::Graph};
use crate::error::Error;

#[derive(Debug, PartialEq)]
/// A LanGRust [File] is composed of functions nodes,
/// types defined by the user and an optional component.
pub struct File {
    /// Program types.
    pub user_defined_types: Vec<UserDefinedType>,
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
    pub fn get_types(self) -> Vec<UserDefinedType> {
        self.user_defined_types
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
    pub fn get_types_functions_nodes(self) -> (Vec<UserDefinedType>, Vec<Function>, Vec<Node>) {
        (self.user_defined_types, self.functions, self.nodes)
    }
    /// Get the location of a LanGRust file.
    pub fn get_location(self) -> Location {
        self.location
    }
    /// Add a type definition to a LanGRust file functions.
    pub fn push_type(&mut self, user_defined_type: UserDefinedType) {
        self.user_defined_types.push(user_defined_type)
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
    ///     statement::Statement, function::Function, location::Location,
    ///     expression::Expression, type_system::Type, equation::Equation, node::Node, file::File,
    ///     scope::Scope, stream_expression::StreamExpression,
    /// };
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
    /// let mut file = File {
    ///     user_defined_types: vec![],
    ///     functions: vec![function],
    ///     nodes: vec![node],
    ///     component: None,
    ///     location: Location::default(),
    /// };
    ///
    /// file.typing(&mut errors).unwrap();
    /// ```
    pub fn typing(&mut self, errors: &mut Vec<Error>) -> Result<(), ()> {
        let File {
            user_defined_types,
            functions,
            nodes,
            component,
            ..
        } = self;

        // create user_types_context
        let mut user_types_context = HashMap::new();
        user_defined_types
            .iter()
            .map(|user_type| match user_type {
                UserDefinedType::Structure { id, location, .. }
                | UserDefinedType::Enumeration { id, location, .. }
                | UserDefinedType::Array { id, location, .. } => user_types_context.insert_unique(
                    id.clone(),
                    user_type.clone(),
                    location.clone(),
                    errors,
                ),
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // resolve undefined types in user_defined_types
        user_defined_types
            .iter_mut()
            .map(|user_type| user_type.resolve_undefined_types(&user_types_context, errors))
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // recreate a user_types_context with resolved types
        let mut user_types_context = HashMap::new();
        user_defined_types
            .iter()
            .map(|user_type| match user_type {
                UserDefinedType::Structure { id, location, .. }
                | UserDefinedType::Enumeration { id, location, .. }
                | UserDefinedType::Array { id, location, .. } => user_types_context.insert_unique(
                    id.clone(),
                    user_type.clone(),
                    location.clone(),
                    errors,
                ),
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // resolve undefined types in nodes
        nodes
            .iter_mut()
            .map(|node| node.resolve_undefined_types(&user_types_context, errors))
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // resolve undefined types in component
        component.as_mut().map_or(Ok(()), |component| {
            Ok(component.resolve_undefined_types(&user_types_context, errors)?)
        })?;

        // resolve undefined types in functions
        functions
            .iter_mut()
            .map(|function| function.resolve_undefined_types(&user_types_context, errors))
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
                    let function_type = inputs.iter().rev().fold(
                        returned_type.clone(),
                        |current_type, (_, input_type)| {
                            Type::Abstract(Box::new(input_type.clone()), Box::new(current_type))
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
            .map(|node| node.typing(&nodes_context, &global_context, &user_types_context, errors))
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // typing component
        component.as_mut().map_or(Ok(()), |component| {
            Ok(component.typing(&nodes_context, &global_context, &user_types_context, errors)?)
        })?;

        // typing functions
        functions
            .iter_mut()
            .map(|function| function.typing(&global_context, &user_types_context, errors))
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()
    }

    /// Generate dependencies graph for every nodes/component.
    ///
    /// # Example
    /// ```rust
    /// use grustine::ast::{
    ///     statement::Statement, function::Function, location::Location,
    ///     expression::Expression, type_system::Type, equation::Equation, node::Node, file::File,
    ///     scope::Scope, stream_expression::StreamExpression,
    /// };
    /// use grustine::common::{color::Color, graph::Graph};
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
    /// let mut file = File {
    ///     user_defined_types: vec![],
    ///     functions: vec![function],
    ///     nodes: vec![node],
    ///     component: None,
    ///     location: Location::default(),
    /// };
    ///
    /// let nodes_graphs = file.generate_dependencies_graphs(&mut errors).unwrap();
    ///
    /// let graph = nodes_graphs.get(&String::from("test")).unwrap();
    ///
    /// let mut control = Graph::new();
    /// control.add_vertex(String::from("o"), Color::Black);
    /// control.add_vertex(String::from("x"), Color::Black);
    /// control.add_vertex(String::from("i"), Color::Black);
    /// control.add_edge(&String::from("x"), String::from("i"), 0);
    /// control.add_edge(&String::from("o"), String::from("x"), 0);
    ///
    /// assert_eq!(*graph, control);
    /// ```
    pub fn generate_dependencies_graphs(
        &self,
        errors: &mut Vec<Error>,
    ) -> Result<HashMap<String, Graph<Color>>, ()> {
        let File {
            nodes, component, ..
        } = self;

        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();

        nodes
            .into_iter()
            .map(|node| {
                let graph = node.create_initialized_graph(errors)?;
                nodes_graphs.insert(node.id.clone(), graph.clone());
                nodes_reduced_graphs.insert(node.id.clone(), graph.clone());
                Ok(())
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;
        component.as_ref().map_or(Ok(()), |component| {
            let graph = component.create_initialized_graph(errors)?;
            nodes_graphs.insert(component.id.clone(), graph.clone());
            nodes_reduced_graphs.insert(component.id.clone(), graph.clone());
            Ok(())
        })?;

        let nodes_context = nodes
            .iter()
            .map(|node| (node.id.clone(), node.clone()))
            .collect::<HashMap<_, _>>();

        nodes
            .into_iter()
            .map(|node| {
                node.add_all_dependencies(
                    &nodes_context,
                    &mut nodes_graphs,
                    &mut nodes_reduced_graphs,
                    errors,
                )
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;
        component.as_ref().map_or(Ok(()), |component| {
            component.add_all_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                errors,
            )
        })?;

        Ok(nodes_graphs)
    }
}

#[cfg(test)]
mod typing {
    use crate::ast::{
        equation::Equation, expression::Expression, file::File, function::Function,
        location::Location, node::Node, scope::Scope, statement::Statement,
        stream_expression::StreamExpression, type_system::Type,
    };

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

        let mut file = File {
            user_defined_types: vec![],
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

        let control = File {
            user_defined_types: vec![],
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

        let mut file = File {
            user_defined_types: vec![],
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

        let control = File {
            user_defined_types: vec![],
            functions: vec![expected_function],
            nodes: vec![expected_node],
            component: Some(expected_component),
            location: Location::default(),
        };

        file.typing(&mut errors).unwrap();

        assert_eq!(file, control);
    }
}
