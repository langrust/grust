use std::collections::HashMap;

use crate::ast::typedef::Typedef;
use crate::common::{
    graph::{color::Color, Graph},
    location::Location,
};
use crate::error::Error;
use crate::hir::{function::Function, node::Node};

#[derive(Debug, PartialEq)]
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
    /// Generate dependencies graph for every nodes/component.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::hir::{
    ///     equation::Equation, expression::Expression, function::Function,
    ///     file::File, node::Node, statement::Statement, stream_expression::StreamExpression,
    /// };
    /// use grustine::common::{
    ///     graph::{color::Color, Graph}, location::Location, scope::Scope, r#type::Type
    /// };
    ///
    /// let mut errors = vec![];
    ///
    /// let node = Node {
    ///     id: String::from("test"),
    ///     is_component: false,
    ///     inputs: vec![(String::from("i"), Type::Integer)],
    ///     unscheduled_equations: HashMap::from([
    ///         (
    ///             String::from("o"),
    ///             Equation {
    ///                 scope: Scope::Output,
    ///                 id: String::from("o"),
    ///                 signal_type: Type::Integer,
    ///                 expression: StreamExpression::SignalCall {
    ///                     id: String::from("x"),
    ///                     typing: Type::Integer,
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
    ///                     typing: Type::Integer,
    ///                     location: Location::default(),
    ///                 },
    ///                 location: Location::default(),
    ///             }
    ///         )
    ///     ]),
    ///     unitary_nodes: HashMap::new(),
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
    ///                 typing: Type::Integer,
    ///                 location: Location::default(),
    ///             },
    ///             location: Location::default(),
    ///         }
    ///     ],
    ///     returned: (
    ///         Type::Integer,
    ///         Expression::Call {
    ///             id: String::from("x"),
    ///             typing: Type::Integer,
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
    /// let nodes_graphs = file.generate_dependency_graphs(&mut errors).unwrap();
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
    pub fn generate_dependency_graphs(
        &self,
        errors: &mut Vec<Error>,
    ) -> Result<HashMap<String, Graph<Color>>, ()> {
        let File {
            nodes, component, ..
        } = self;

        // initialize dictionaries for graphs
        let mut nodes_graphs = HashMap::new();
        let mut nodes_reduced_graphs = HashMap::new();

        // initialize every nodes' graphs
        nodes
            .into_iter()
            .map(|node| {
                let graph = node.create_initialized_graph();
                nodes_graphs.insert(node.id.clone(), graph.clone());
                nodes_reduced_graphs.insert(node.id.clone(), graph.clone());
                Ok(())
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // optional component's graph initialization
        component.as_ref().map_or(Ok(()), |component| {
            let graph = component.create_initialized_graph();
            nodes_graphs.insert(component.id.clone(), graph.clone());
            nodes_reduced_graphs.insert(component.id.clone(), graph.clone());
            Ok(())
        })?;

        // creates nodes context: nodes dictionary
        let nodes_context = nodes
            .iter()
            .map(|node| (node.id.clone(), node.clone()))
            .collect::<HashMap<_, _>>();

        // every nodes complete their dependencies graphs
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

        // optional component completes its dependencies graph
        component.as_ref().map_or(Ok(()), |component| {
            component.add_all_dependencies(
                &nodes_context,
                &mut nodes_graphs,
                &mut nodes_reduced_graphs,
                errors,
            )
        })?;

        // return direct dependencies graphs
        Ok(nodes_graphs)
    }

    /// Generate unitary nodes.
    ///
    /// It also changes node application expressions into unitary node application
    /// and removes unused inputs from those unitary node application.
    ///
    /// # Example
    ///
    /// Let be a node `my_node` and a node `other_node` as follows:
    ///
    /// ```GR
    /// node my_node(x: int, y: int) {
    ///     out o1: int = x+y;
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int, g: int) {
    ///     out x: int = my_node(y, v).o1;
    ///     y: int = my_node(g-1, v).o2;
    /// }
    /// ```
    ///
    /// The generated unitary nodes are the following:
    ///
    /// ```GR
    /// node my_node(x: int, y: int).o1 {
    ///     out o1: int = x+y;
    /// }
    /// node my_node(y: int).o2 {
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int) {           // g is then unused and will raise an error
    ///     out x: int = my_node(y, v).o1;
    ///     y: int = my_node(v).o2;
    /// }
    /// ```
    pub fn generate_unitary_nodes(&mut self, errors: &mut Vec<Error>) -> Result<(), ()> {
        // generate dependency graph
        let mut graphs = self.generate_dependencies_graphs(errors)?;

        // unitary nodes computations, it induces schedulings of the node
        self.nodes
            .iter_mut()
            .map(|node| {
                let graph = graphs.get_mut(&node.id).unwrap();
                node.generate_unitary_nodes(graph, errors)
            })
            .collect::<Vec<Result<(), ()>>>()
            .into_iter()
            .collect::<Result<(), ()>>()?;

        // get, for each unitary node, initial node's inputs
        // that are used by the unitary node
        let unitary_nodes_used_inputs = self
            .nodes
            .iter()
            .map(|node| {
                (
                    node.id.clone(),
                    node.unitary_nodes
                        .iter()
                        .map(|(output, unitary_node)| {
                            (
                                output.clone(),
                                node.inputs
                                    .iter()
                                    .map(|input| unitary_node.inputs.contains(input))
                                    .collect::<Vec<bool>>(),
                            )
                        })
                        .collect::<HashMap<String, Vec<bool>>>(),
                )
            })
            .collect::<HashMap<String, HashMap<String, Vec<bool>>>>();

        // change node application to unitary node application
        self.nodes.iter_mut().for_each(|node| {
            node.unitary_nodes.values_mut().for_each(|unitary_node| {
                unitary_node
                    .scheduled_equations
                    .iter_mut()
                    .for_each(|equation| {
                        equation
                            .expression
                            .change_node_application_into_unitary_node_application(
                                &unitary_nodes_used_inputs,
                            )
                    })
            })
        });

        Ok(())
    }

    /// Normalize HIR file.
    ///
    /// Normalize all nodes of a file as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// # Example
    ///
    /// ```GR
    /// function test(i: int) -> int {
    ///     let x: int = i;
    ///     return x;
    /// }
    /// node my_node(x: int, y: int) {
    ///     out o: int = x*y;
    /// }
    /// node other_node(x: int, y: int) {
    ///     out o: int = x*y;
    /// }
    /// node test(s: int, v: int, g: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    ///     out y: int = other_node(g-1, v).o;
    /// }
    /// ```
    ///
    /// The above node contains the following unitary nodes:
    ///
    /// ```GR
    /// node test_x(s: int, v: int) {
    ///     out x: int = 1 + my_node(s, v*2).o;
    /// }
    /// node test_y(v: int, g: int) {
    ///     out y: int = other_node(g-1, v).o;
    /// }
    /// ```
    ///
    /// Which are normalized into:
    ///
    /// ```GR
    /// node test_x(s: int, v: int) {
    ///     x_1: int = v*2;
    ///     x_2: int = my_node(s, x_1).o;
    ///     out x: int = 1 + x_2;
    /// }
    /// node test_y(v: int, g: int) {
    ///     x: int = g-1;
    ///     out y: int = other_node(x_1, v).o;
    /// }
    /// ```
    ///
    /// This example is tested in source.
    pub fn normalize(&mut self, errors: &mut Vec<Error>) -> Result<(), ()> {
        self.generate_unitary_nodes(errors)?;

        self.nodes.iter_mut().for_each(|node| node.normalize());
        Ok(())
    }
}

#[cfg(test)]
mod generate_unitary_nodes {
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        equation::Equation, expression::Expression, file::File, function::Function, memory::Memory,
        node::Node, statement::Statement, stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    };
    use std::collections::HashMap;

    #[test]
    fn should_generate_unitary_nodes_from_nodes() {
        let mut errors = vec![];

        // my_node(x: int, y: int) { out o1: int = x+y; out o2: int = 2*y; }
        let node = Node {
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("+"),
                                typing: Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                ),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("x"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("*"),
                                typing: Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                ),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::Constant {
                                    constant: Constant::Integer(2),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
        };
        let mut file = File {
            typedefs: vec![],
            functions: vec![],
            nodes: vec![node],
            component: None,
            location: Location::default(),
        };
        file.generate_unitary_nodes(&mut errors).unwrap();

        let node_control = Node {
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("+"),
                                typing: Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                ),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("x"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("*"),
                                typing: Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                ),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::Constant {
                                    constant: Constant::Integer(2),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::from([
                (
                    String::from("o1"),
                    UnitaryNode {
                        node_id: String::from("my_node"),
                        output_id: String::from("o1"),
                        inputs: vec![
                            (String::from("x"), Type::Integer),
                            (String::from("y"), Type::Integer),
                        ],
                        scheduled_equations: vec![Equation {
                            scope: Scope::Output,
                            id: String::from("o1"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: String::from("+"),
                                    typing: Type::Abstract(
                                        vec![Type::Integer, Type::Integer],
                                        Box::new(Type::Integer),
                                    ),
                                    location: Location::default(),
                                },
                                inputs: vec![
                                    StreamExpression::SignalCall {
                                        id: String::from("x"),
                                        typing: Type::Integer,
                                        location: Location::default(),
                                    },
                                    StreamExpression::SignalCall {
                                        id: String::from("y"),
                                        typing: Type::Integer,
                                        location: Location::default(),
                                    },
                                ],
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                            location: Location::default(),
                        }],
                        memory: Memory::new(),
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    UnitaryNode {
                        node_id: String::from("my_node"),
                        output_id: String::from("o2"),
                        inputs: vec![(String::from("y"), Type::Integer)],
                        scheduled_equations: vec![Equation {
                            scope: Scope::Output,
                            id: String::from("o2"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: String::from("*"),
                                    typing: Type::Abstract(
                                        vec![Type::Integer, Type::Integer],
                                        Box::new(Type::Integer),
                                    ),
                                    location: Location::default(),
                                },
                                inputs: vec![
                                    StreamExpression::Constant {
                                        constant: Constant::Integer(2),
                                        typing: Type::Integer,
                                        location: Location::default(),
                                    },
                                    StreamExpression::SignalCall {
                                        id: String::from("y"),
                                        typing: Type::Integer,
                                        location: Location::default(),
                                    },
                                ],
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                            location: Location::default(),
                        }],
                        memory: Memory::new(),
                        location: Location::default(),
                    },
                ),
            ]),
            location: Location::default(),
        };
        let control = File {
            typedefs: vec![],
            functions: vec![],
            nodes: vec![node_control],
            component: None,
            location: Location::default(),
        };
        assert_eq!(file, control);
    }

    #[test]
    fn should_change_node_application_to_unitary_node_application() {
        let mut errors = vec![];

        // my_node(x: int, y: int) { out o: int = x*y; }
        let my_node = Node {
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                Equation {
                    scope: Scope::Output,
                    id: String::from("o"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("*"),
                            typing: Type::Abstract(
                                vec![Type::Integer, Type::Integer],
                                Box::new(Type::Integer),
                            ),
                            location: Location::default(),
                        },
                        inputs: vec![
                            StreamExpression::SignalCall {
                                id: String::from("x"),
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                            StreamExpression::SignalCall {
                                id: String::from("y"),
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                        ],
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
            )]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
        };
        // other_node(x: int, y: int) { out o1: int = x+y; out o2: int = 2*y; }
        let other_node = Node {
            id: String::from("other_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("+"),
                                typing: Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                ),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("x"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("*"),
                                typing: Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                ),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::Constant {
                                    constant: Constant::Integer(2),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
        };
        // out x: int = 1 + my_node(s, v*2).o
        let equation_1 = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: String::from("+"),
                    typing: Type::Abstract(
                        vec![Type::Integer, Type::Integer],
                        Box::new(Type::Integer),
                    ),
                    location: Location::default(),
                },
                inputs: vec![
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    StreamExpression::NodeApplication {
                        node: String::from("my_node"),
                        inputs: vec![
                            StreamExpression::SignalCall {
                                id: String::from("s"),
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                            StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: String::from("*2"),
                                    typing: Type::Abstract(
                                        vec![Type::Integer],
                                        Box::new(Type::Integer),
                                    ),
                                    location: Location::default(),
                                },
                                inputs: vec![StreamExpression::SignalCall {
                                    id: String::from("v"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                }],
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                        ],
                        signal: String::from("o"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
            },
            location: Location::default(),
        };
        // out y: int = other_node(g-1, v).o1
        let equation_2 = Equation {
            scope: Scope::Output,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::NodeApplication {
                node: String::from("other_node"),
                inputs: vec![
                    StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("-1"),
                            typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            id: String::from("g"),
                            typing: Type::Integer,
                            location: Location::default(),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    StreamExpression::SignalCall {
                        id: String::from("v"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ],
                signal: String::from("o1"),
                typing: Type::Integer,
                location: Location::default(),
            },
            location: Location::default(),
        };
        // out z: int = other_node(g-1, v).o2
        let equation_3 = Equation {
            scope: Scope::Output,
            id: String::from("z"),
            signal_type: Type::Integer,
            expression: StreamExpression::NodeApplication {
                node: String::from("other_node"),
                inputs: vec![
                    StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("-1"),
                            typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            id: String::from("g"),
                            typing: Type::Integer,
                            location: Location::default(),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    StreamExpression::SignalCall {
                        id: String::from("v"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ],
                signal: String::from("o2"),
                typing: Type::Integer,
                location: Location::default(),
            },
            location: Location::default(),
        };
        let node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1.clone()),
                (String::from("y"), equation_2.clone()),
                (String::from("z"), equation_3.clone()),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
        };
        let function = Function {
            id: String::from("my_function"),
            inputs: vec![(String::from("i"), Type::Integer)],
            statements: vec![Statement {
                id: String::from("x"),
                element_type: Type::Integer,
                expression: Expression::Call {
                    id: String::from("i"),
                    typing: Type::Integer,
                    location: Location::default(),
                },
                location: Location::default(),
            }],
            returned: (
                Type::Integer,
                Expression::Call {
                    id: String::from("x"),
                    typing: Type::Integer,
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };
        let mut file = File {
            typedefs: vec![],
            functions: vec![function],
            nodes: vec![my_node, other_node, node],
            component: None,
            location: Location::default(),
        };
        file.generate_unitary_nodes(&mut errors).unwrap();

        // my_node(x: int, y: int) { out o: int = x*y; }
        let my_node = Node {
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                Equation {
                    scope: Scope::Output,
                    id: String::from("o"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("*"),
                            typing: Type::Abstract(
                                vec![Type::Integer, Type::Integer],
                                Box::new(Type::Integer),
                            ),
                            location: Location::default(),
                        },
                        inputs: vec![
                            StreamExpression::SignalCall {
                                id: String::from("x"),
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                            StreamExpression::SignalCall {
                                id: String::from("y"),
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                        ],
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
            )]),
            unitary_nodes: HashMap::from([(
                String::from("o"),
                UnitaryNode {
                    node_id: String::from("my_node"),
                    output_id: String::from("o"),
                    inputs: vec![
                        (String::from("x"), Type::Integer),
                        (String::from("y"), Type::Integer),
                    ],
                    scheduled_equations: vec![Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("*"),
                                typing: Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                ),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("x"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    }],
                    memory: Memory::new(),
                    location: Location::default(),
                },
            )]),
            location: Location::default(),
        };
        // other_node(x: int, y: int) { out o1: int = x+y; out o2: int = 2*y; }
        let other_node = Node {
            id: String::from("other_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("+"),
                                typing: Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                ),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("x"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("*"),
                                typing: Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                ),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::Constant {
                                    constant: Constant::Integer(2),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::from([
                (
                    String::from("o1"),
                    UnitaryNode {
                        node_id: String::from("other_node"),
                        output_id: String::from("o1"),
                        inputs: vec![
                            (String::from("x"), Type::Integer),
                            (String::from("y"), Type::Integer),
                        ],
                        scheduled_equations: vec![Equation {
                            scope: Scope::Output,
                            id: String::from("o1"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: String::from("+"),
                                    typing: Type::Abstract(
                                        vec![Type::Integer, Type::Integer],
                                        Box::new(Type::Integer),
                                    ),
                                    location: Location::default(),
                                },
                                inputs: vec![
                                    StreamExpression::SignalCall {
                                        id: String::from("x"),
                                        typing: Type::Integer,
                                        location: Location::default(),
                                    },
                                    StreamExpression::SignalCall {
                                        id: String::from("y"),
                                        typing: Type::Integer,
                                        location: Location::default(),
                                    },
                                ],
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                            location: Location::default(),
                        }],
                        memory: Memory::new(),
                        location: Location::default(),
                    },
                ),
                (
                    String::from("o2"),
                    UnitaryNode {
                        node_id: String::from("other_node"),
                        output_id: String::from("o2"),
                        inputs: vec![(String::from("y"), Type::Integer)],
                        scheduled_equations: vec![Equation {
                            scope: Scope::Output,
                            id: String::from("o2"),
                            signal_type: Type::Integer,
                            expression: StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: String::from("*"),
                                    typing: Type::Abstract(
                                        vec![Type::Integer, Type::Integer],
                                        Box::new(Type::Integer),
                                    ),
                                    location: Location::default(),
                                },
                                inputs: vec![
                                    StreamExpression::Constant {
                                        constant: Constant::Integer(2),
                                        typing: Type::Integer,
                                        location: Location::default(),
                                    },
                                    StreamExpression::SignalCall {
                                        id: String::from("y"),
                                        typing: Type::Integer,
                                        location: Location::default(),
                                    },
                                ],
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                            location: Location::default(),
                        }],
                        memory: Memory::new(),
                        location: Location::default(),
                    },
                ),
            ]),
            location: Location::default(),
        };
        // out x: int = 1 + my_node(s, v*2).o
        let unitary_equation_1 = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: String::from("+"),
                    typing: Type::Abstract(
                        vec![Type::Integer, Type::Integer],
                        Box::new(Type::Integer),
                    ),
                    location: Location::default(),
                },
                inputs: vec![
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    StreamExpression::UnitaryNodeApplication {
                        node: String::from("my_node"),
                        inputs: vec![
                            StreamExpression::SignalCall {
                                id: String::from("s"),
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                            StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: String::from("*2"),
                                    typing: Type::Abstract(
                                        vec![Type::Integer],
                                        Box::new(Type::Integer),
                                    ),
                                    location: Location::default(),
                                },
                                inputs: vec![StreamExpression::SignalCall {
                                    id: String::from("v"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                }],
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                        ],
                        signal: String::from("o"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
            },
            location: Location::default(),
        };
        let unitary_node_1 = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            scheduled_equations: vec![unitary_equation_1],
            memory: Memory::new(),
            location: Location::default(),
        };
        // out y: int = other_node(g-1, v).o1
        let unitary_equation_2 = Equation {
            scope: Scope::Output,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                node: String::from("other_node"),
                inputs: vec![
                    StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("-1"),
                            typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            id: String::from("g"),
                            typing: Type::Integer,
                            location: Location::default(),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    StreamExpression::SignalCall {
                        id: String::from("v"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ],
                signal: String::from("o1"),
                typing: Type::Integer,
                location: Location::default(),
            },
            location: Location::default(),
        };
        let unitary_node_2 = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("y"),
            inputs: vec![
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            scheduled_equations: vec![unitary_equation_2],
            memory: Memory::new(),
            location: Location::default(),
        };
        // out z: int = other_node(g-1, v).o2
        let unitary_equation_3 = Equation {
            scope: Scope::Output,
            id: String::from("z"),
            signal_type: Type::Integer,
            expression: StreamExpression::UnitaryNodeApplication {
                node: String::from("other_node"),
                inputs: vec![StreamExpression::SignalCall {
                    id: String::from("v"),
                    typing: Type::Integer,
                    location: Location::default(),
                }],
                signal: String::from("o2"),
                typing: Type::Integer,
                location: Location::default(),
            },
            location: Location::default(),
        };
        let unitary_node_3 = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("z"),
            inputs: vec![(String::from("v"), Type::Integer)],
            scheduled_equations: vec![unitary_equation_3],
            memory: Memory::new(),
            location: Location::default(),
        };
        let node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1),
                (String::from("y"), equation_2),
                (String::from("z"), equation_3),
            ]),
            unitary_nodes: HashMap::from([
                (String::from("x"), unitary_node_1),
                (String::from("y"), unitary_node_2),
                (String::from("z"), unitary_node_3),
            ]),
            location: Location::default(),
        };
        let function = Function {
            id: String::from("my_function"),
            inputs: vec![(String::from("i"), Type::Integer)],
            statements: vec![Statement {
                id: String::from("x"),
                element_type: Type::Integer,
                expression: Expression::Call {
                    id: String::from("i"),
                    typing: Type::Integer,
                    location: Location::default(),
                },
                location: Location::default(),
            }],
            returned: (
                Type::Integer,
                Expression::Call {
                    id: String::from("x"),
                    typing: Type::Integer,
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };
        let control = File {
            typedefs: vec![],
            functions: vec![function],
            nodes: vec![my_node, other_node, node],
            component: None,
            location: Location::default(),
        };
        assert_eq!(file, control);
    }
}

#[cfg(test)]
mod normalize {
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        equation::Equation, expression::Expression, file::File, function::Function, memory::Memory,
        node::Node, statement::Statement, stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    };
    use std::collections::HashMap;

    #[test]
    fn should_normalize_nodes_in_file() {
        let mut errors = vec![];

        let my_node = Node {
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                Equation {
                    scope: Scope::Output,
                    id: String::from("o"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("*"),
                            typing: Type::Abstract(
                                vec![Type::Integer, Type::Integer],
                                Box::new(Type::Integer),
                            ),
                            location: Location::default(),
                        },
                        inputs: vec![
                            StreamExpression::SignalCall {
                                id: String::from("x"),
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                            StreamExpression::SignalCall {
                                id: String::from("y"),
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                        ],
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
            )]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
        };

        let other_node = Node {
            id: String::from("other_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                Equation {
                    scope: Scope::Output,
                    id: String::from("o"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("*"),
                            typing: Type::Abstract(
                                vec![Type::Integer, Type::Integer],
                                Box::new(Type::Integer),
                            ),
                            location: Location::default(),
                        },
                        inputs: vec![
                            StreamExpression::SignalCall {
                                id: String::from("x"),
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                            StreamExpression::SignalCall {
                                id: String::from("y"),
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                        ],
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
            )]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
        };

        let equation_1 = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::MapApplication {
                function_expression: Expression::Call {
                    id: String::from("+"),
                    typing: Type::Abstract(
                        vec![Type::Integer, Type::Integer],
                        Box::new(Type::Integer),
                    ),
                    location: Location::default(),
                },
                inputs: vec![
                    StreamExpression::Constant {
                        constant: Constant::Integer(1),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    StreamExpression::NodeApplication {
                        node: String::from("my_node"),
                        inputs: vec![
                            StreamExpression::SignalCall {
                                id: String::from("s"),
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                            StreamExpression::MapApplication {
                                function_expression: Expression::Call {
                                    id: String::from("*2"),
                                    typing: Type::Abstract(
                                        vec![Type::Integer],
                                        Box::new(Type::Integer),
                                    ),
                                    location: Location::default(),
                                },
                                inputs: vec![StreamExpression::SignalCall {
                                    id: String::from("v"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                }],
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                        ],
                        signal: String::from("o"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ],
                typing: Type::Integer,
                location: Location::default(),
            },
            location: Location::default(),
        };
        let equation_2 = Equation {
            scope: Scope::Output,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::NodeApplication {
                node: String::from("other_node"),
                inputs: vec![
                    StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("-1"),
                            typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            id: String::from("g"),
                            typing: Type::Integer,
                            location: Location::default(),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    StreamExpression::SignalCall {
                        id: String::from("v"),
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                ],
                signal: String::from("o"),
                typing: Type::Integer,
                location: Location::default(),
            },
            location: Location::default(),
        };
        let node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1.clone()),
                (String::from("y"), equation_2.clone()),
            ]),
            unitary_nodes: HashMap::new(),
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
                    typing: Type::Integer,
                    location: Location::default(),
                },
                location: Location::default(),
            }],
            returned: (
                Type::Integer,
                Expression::Call {
                    id: String::from("x"),
                    typing: Type::Integer,
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };
        let mut file = File {
            typedefs: vec![],
            functions: vec![function],
            nodes: vec![my_node, other_node, node],
            component: None,
            location: Location::default(),
        };
        file.normalize(&mut errors).unwrap();

        let my_node = Node {
            id: String::from("my_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                Equation {
                    scope: Scope::Output,
                    id: String::from("o"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("*"),
                            typing: Type::Abstract(
                                vec![Type::Integer, Type::Integer],
                                Box::new(Type::Integer),
                            ),
                            location: Location::default(),
                        },
                        inputs: vec![
                            StreamExpression::SignalCall {
                                id: String::from("x"),
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                            StreamExpression::SignalCall {
                                id: String::from("y"),
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                        ],
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
            )]),
            unitary_nodes: HashMap::from([(
                String::from("o"),
                UnitaryNode {
                    node_id: String::from("my_node"),
                    output_id: String::from("o"),
                    inputs: vec![
                        (String::from("x"), Type::Integer),
                        (String::from("y"), Type::Integer),
                    ],
                    scheduled_equations: vec![Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("*"),
                                typing: Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                ),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("x"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    }],
                    memory: Memory::new(),
                    location: Location::default(),
                },
            )]),
            location: Location::default(),
        };

        let other_node = Node {
            id: String::from("other_node"),
            is_component: false,
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                Equation {
                    scope: Scope::Output,
                    id: String::from("o"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("*"),
                            typing: Type::Abstract(
                                vec![Type::Integer, Type::Integer],
                                Box::new(Type::Integer),
                            ),
                            location: Location::default(),
                        },
                        inputs: vec![
                            StreamExpression::SignalCall {
                                id: String::from("x"),
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                            StreamExpression::SignalCall {
                                id: String::from("y"),
                                typing: Type::Integer,
                                location: Location::default(),
                            },
                        ],
                        typing: Type::Integer,
                        location: Location::default(),
                    },
                    location: Location::default(),
                },
            )]),
            unitary_nodes: HashMap::from([(
                String::from("o"),
                UnitaryNode {
                    node_id: String::from("other_node"),
                    output_id: String::from("o"),
                    inputs: vec![
                        (String::from("x"), Type::Integer),
                        (String::from("y"), Type::Integer),
                    ],
                    scheduled_equations: vec![Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("*"),
                                typing: Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                ),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("x"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        location: Location::default(),
                    }],
                    memory: Memory::new(),
                    location: Location::default(),
                },
            )]),
            location: Location::default(),
        };

        let equations_1 = vec![
            Equation {
                scope: Scope::Local,
                id: String::from("x_1"),
                signal_type: Type::Integer,
                expression: StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("*2"),
                        typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("v"),
                        typing: Type::Integer,
                        location: Location::default(),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                },
                location: Location::default(),
            },
            Equation {
                scope: Scope::Local,
                id: String::from("x_2"),
                signal_type: Type::Integer,
                expression: StreamExpression::UnitaryNodeApplication {
                    node: String::from("my_node"),
                    inputs: vec![
                        StreamExpression::SignalCall {
                            id: String::from("s"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        StreamExpression::SignalCall {
                            id: String::from("x_1"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                    ],
                    signal: String::from("o"),
                    typing: Type::Integer,
                    location: Location::default(),
                },
                location: Location::default(),
            },
            Equation {
                scope: Scope::Output,
                id: String::from("x"),
                signal_type: Type::Integer,
                expression: StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("+"),
                        typing: Type::Abstract(
                            vec![Type::Integer, Type::Integer],
                            Box::new(Type::Integer),
                        ),
                        location: Location::default(),
                    },
                    inputs: vec![
                        StreamExpression::Constant {
                            constant: Constant::Integer(1),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        StreamExpression::SignalCall {
                            id: String::from("x_2"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                    ],
                    typing: Type::Integer,
                    location: Location::default(),
                },
                location: Location::default(),
            },
        ];
        let unitary_node_1 = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
            ],
            scheduled_equations: equations_1,
            memory: Memory::new(),
            location: Location::default(),
        };
        let equations_2 = vec![
            Equation {
                scope: Scope::Local,
                id: String::from("x"),
                signal_type: Type::Integer,
                expression: StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("-1"),
                        typing: Type::Abstract(vec![Type::Integer], Box::new(Type::Integer)),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("g"),
                        typing: Type::Integer,
                        location: Location::default(),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                },
                location: Location::default(),
            },
            Equation {
                scope: Scope::Output,
                id: String::from("y"),
                signal_type: Type::Integer,
                expression: StreamExpression::UnitaryNodeApplication {
                    node: String::from("other_node"),
                    inputs: vec![
                        StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                        StreamExpression::SignalCall {
                            id: String::from("v"),
                            typing: Type::Integer,
                            location: Location::default(),
                        },
                    ],
                    signal: String::from("o"),
                    typing: Type::Integer,
                    location: Location::default(),
                },
                location: Location::default(),
            },
        ];
        let unitary_node_2 = UnitaryNode {
            node_id: String::from("test"),
            output_id: String::from("y"),
            inputs: vec![
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            scheduled_equations: equations_2,
            memory: Memory::new(),
            location: Location::default(),
        };
        let node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("s"), Type::Integer),
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1),
                (String::from("y"), equation_2),
            ]),
            unitary_nodes: HashMap::from([
                (String::from("x"), unitary_node_1),
                (String::from("y"), unitary_node_2),
            ]),
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
                    typing: Type::Integer,
                    location: Location::default(),
                },
                location: Location::default(),
            }],
            returned: (
                Type::Integer,
                Expression::Call {
                    id: String::from("x"),
                    typing: Type::Integer,
                    location: Location::default(),
                },
            ),
            location: Location::default(),
        };
        let control = File {
            typedefs: vec![],
            functions: vec![function],
            nodes: vec![my_node, other_node, node],
            component: None,
            location: Location::default(),
        };
        assert_eq!(file, control);
    }
}
