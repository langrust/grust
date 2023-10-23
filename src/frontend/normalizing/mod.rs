mod inlining;
mod normal_form;
mod scheduling;
mod unitary_node;

use crate::{error::Error, hir::file::File};

impl File {
    /// Normalize HIR nodes in file.
    ///
    /// This is a chain of the following computations:
    /// - unitary nodes generation (check also that all signals are used)
    /// - inlining unitary node calls when needed (shifted causality loops)
    /// - scheduling unitary nodes
    /// - normalizing unitary node application
    ///
    /// # Example
    ///
    /// Let be a node `my_node` and a node `other_node` as follows:
    ///
    /// ```GR
    /// node mem(i: int) {
    ///     out o: int = 0 fby i;
    /// }
    ///
    /// node my_node(x: int, y: int) {
    ///     out o1: int = x+y;
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int, g: int) {
    ///     out x: int = my_node(y, v).o1;
    ///     y: int = 1 + my_node(g-1, v-1).o2;
    ///     out z: int = mem(z).o;
    /// }
    /// ```
    ///
    /// ## Generate unitary nodes
    ///
    /// The generated unitary nodes are the following:
    ///
    /// ```GR
    /// node mem(i: int).o {
    ///     out o: int = 0 fby i;
    /// }
    ///
    /// node my_node(x: int, y: int).o1 {
    ///     out o1: int = x+y;
    /// }
    /// node my_node(y: int).o2 {
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int).x {
    ///     out x: int = my_node(y, v).o1;
    ///     y: int = 1 + my_node(v-1).o2;
    /// }
    /// node other_node().z {
    ///     out z: int = mem(z).o;
    /// }
    /// ```
    ///
    /// But `g` is then unused, this will raise an error and stop the compilation.
    ///
    /// ## Inlining unitary nodes
    ///
    /// Suppose that we did not write `g` in the code and that the compilation
    /// succeeded the unitary node generation step. The inlining step will modify
    /// the unitary nodes as follows:
    ///
    /// ```GR
    /// node mem(i: int).o {
    ///     out o: int = 0 fby i;
    /// }
    ///
    /// node my_node(x: int, y: int).o1 {
    ///     out o1: int = x+y;
    /// }
    /// node my_node(y: int).o2 {
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int).x {
    ///     out x: int = my_node(y, v).o1;
    ///     y: int = 1 + my_node(v-1).o2;
    /// }
    /// node other_node().z {
    ///     out z: int = 0 fby z;
    /// }
    /// ```
    ///
    /// In this example, `other_node` calls `mem` with the same input and output signal.
    /// There is no causality loop, `o` depends on the memory of `i`.
    ///
    /// We need to inline the code, the output `z` is defined before the input `z`,
    /// which can not be computed by a function call.
    ///
    /// ## Scheduling unitary nodes
    ///
    /// The scheduling step will order the equations of the unitary nodes.
    /// In our example, this will modify the code as bellow.
    ///
    /// ```GR
    /// node mem(i: int).o {
    ///     out o: int = 0 fby i;
    /// }
    ///
    /// node my_node(x: int, y: int).o1 {
    ///     out o1: int = x+y;
    /// }
    /// node my_node(y: int).o2 {
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int).x {
    ///     y: int = 1 + my_node(v-1).o2;         // y is before x now
    ///     out x: int = my_node(y, v).o1;
    /// }
    /// node other_node().z {
    ///     out z: int = 0 fby z;
    /// }
    /// ```
    ///
    /// ## Normal for of unitary nodes
    ///
    /// The last step is the final normal form of the unitary nodes.
    /// The normal form of an unitary node is as follows:
    /// - node application can only append at root expression
    /// - node application inputs are signal calls
    ///
    /// This correspond in our example to the following code:
    /// ```GR
    /// node mem(i: int).o {
    ///     out o: int = 0 fby i;
    /// }
    ///
    /// node my_node(x: int, y: int).o1 {
    ///     out o1: int = x+y;
    /// }
    /// node my_node(y: int).o2 {
    ///     out o2: int = 2*y;
    /// }
    ///
    /// node other_node(v: int).x {
    ///     x_1: int = v-1;             // x_1 was created
    ///     x_2: int = my_node(x_1).o2; // x_2 was created
    ///     y: int = 1 + x_2;
    ///     out x: int = my_node(y, v).o1;
    /// }
    /// node other_node().z {
    ///     out z: int = 0 fby z;
    /// }
    /// ```
    pub fn normalize(&mut self, errors: &mut Vec<Error>) -> Result<(), ()> {
        self.generate_unitary_nodes(errors)?; // check that all signals are used
        self.inline_when_needed();
        self.schedule();
        self.normal_form();
        Ok(())
    }
}

#[cfg(test)]
mod normalize {
    use once_cell::sync::OnceCell;

    use crate::ast::expression::Expression;
    use crate::common::graph::{color::Color, Graph};
    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, file::File, memory::Memory, node::Node,
        stream_expression::StreamExpression, unitary_node::UnitaryNode,
    };
    use std::collections::HashMap;

    #[test]
    fn should_raise_error_for_unused_signals() {
        let mut errors = vec![];

        // node my_node(x: int, y: int) {
        //     out o1: int = x+y;
        //     out o2: int = 2*y;
        // }
        let mut my_node_graph = Graph::new();
        my_node_graph.add_vertex(String::from("x"), Color::Black);
        my_node_graph.add_vertex(String::from("y"), Color::Black);
        my_node_graph.add_vertex(String::from("o1"), Color::Black);
        my_node_graph.add_vertex(String::from("o2"), Color::Black);
        my_node_graph.add_edge(&String::from("o1"), String::from("x"), 0);
        my_node_graph.add_edge(&String::from("o1"), String::from("y"), 0);
        my_node_graph.add_edge(&String::from("o2"), String::from("y"), 0);
        let my_node = Node {
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
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("x"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![
                                (String::from("x"), 0),
                                (String::from("y"), 0),
                            ]),
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
                                id: String::from("2*"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![StreamExpression::SignalCall {
                                id: String::from("y"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                            }],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::from(my_node_graph),
        };

        // node mem(i: int) {
        //     out o: int = 0 fby i;
        // }
        let mut mem_graph = Graph::new();
        mem_graph.add_vertex(String::from("i"), Color::Black);
        mem_graph.add_vertex(String::from("o"), Color::Black);
        mem_graph.add_edge(&String::from("o"), String::from("i"), 1);
        let mem = Node {
            id: String::from("mem"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                Equation {
                    scope: Scope::Output,
                    id: String::from("o"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::SignalCall {
                            id: String::from("i"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i"), 1)]),
                    },
                    location: Location::default(),
                },
            )]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::from(mem_graph),
        };

        // node other_node(v: int, g: int) {
        //     out x: int = my_node(y, v).o1;
        //     y: int = 1 + my_node(g-1, v-1).o2;
        //     out z: int = mem(z).o;
        // }
        let mut other_node_graph = Graph::new();
        other_node_graph.add_vertex(String::from("v"), Color::Black);
        other_node_graph.add_vertex(String::from("g"), Color::Black);
        other_node_graph.add_vertex(String::from("x"), Color::Black);
        other_node_graph.add_vertex(String::from("y"), Color::Black);
        other_node_graph.add_vertex(String::from("z"), Color::Black);
        other_node_graph.add_edge(&String::from("x"), String::from("y"), 0);
        other_node_graph.add_edge(&String::from("x"), String::from("v"), 0);
        other_node_graph.add_edge(&String::from("y"), String::from("v"), 0);
        other_node_graph.add_edge(&String::from("z"), String::from("z"), 1);
        let other_node = Node {
            id: String::from("other_node"),
            is_component: false,
            inputs: vec![
                (String::from("v"), Type::Integer),
                (String::from("g"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("x"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("x"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::NodeApplication {
                            node: String::from("my_node"),
                            signal: String::from("o1"),
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("v"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![
                                (String::from("y"), 0),
                                (String::from("v"), 0),
                            ]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("y"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("1+"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![StreamExpression::NodeApplication {
                                node: String::from("my_node"),
                                signal: String::from("o2"),
                                inputs: vec![
                                    StreamExpression::MapApplication {
                                        function_expression: Expression::Call {
                                            id: String::from("-1"),
                                            typing: Some(Type::Abstract(
                                                vec![Type::Integer],
                                                Box::new(Type::Integer),
                                            )),
                                            location: Location::default(),
                                        },
                                        inputs: vec![StreamExpression::SignalCall {
                                            id: String::from("g"),
                                            typing: Type::Integer,
                                            location: Location::default(),
                                            dependencies: Dependencies::from(vec![(
                                                String::from("g"),
                                                0,
                                            )]),
                                        }],
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::from(vec![(
                                            String::from("g"),
                                            0,
                                        )]),
                                    },
                                    StreamExpression::MapApplication {
                                        function_expression: Expression::Call {
                                            id: String::from("-1"),
                                            typing: Some(Type::Abstract(
                                                vec![Type::Integer],
                                                Box::new(Type::Integer),
                                            )),
                                            location: Location::default(),
                                        },
                                        inputs: vec![StreamExpression::SignalCall {
                                            id: String::from("v"),
                                            typing: Type::Integer,
                                            location: Location::default(),
                                            dependencies: Dependencies::from(vec![(
                                                String::from("v"),
                                                0,
                                            )]),
                                        }],
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::from(vec![(
                                            String::from("v"),
                                            0,
                                        )]),
                                    },
                                ],
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                            }],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("z"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("z"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::NodeApplication {
                            node: String::from("mem"),
                            signal: String::from("o"),
                            inputs: vec![StreamExpression::SignalCall {
                                id: String::from("z"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("z"), 0)]),
                            }],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("z"), 1)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::from(other_node_graph),
        };

        let mut file = File {
            typedefs: vec![],
            functions: vec![],
            nodes: vec![mem, my_node, other_node],
            component: None,
            location: Location::default(),
        };

        file.normalize(&mut errors).unwrap_err()
    }

    #[test]
    fn should_normalize_if_all_signals_are_used() {
        let mut errors = vec![];

        // node my_node(x: int, y: int) {
        //     out o1: int = x+y;
        //     out o2: int = 2*y;
        // }
        let mut my_node_graph = Graph::new();
        my_node_graph.add_vertex(String::from("x"), Color::Black);
        my_node_graph.add_vertex(String::from("y"), Color::Black);
        my_node_graph.add_vertex(String::from("o1"), Color::Black);
        my_node_graph.add_vertex(String::from("o2"), Color::Black);
        my_node_graph.add_edge(&String::from("o1"), String::from("x"), 0);
        my_node_graph.add_edge(&String::from("o1"), String::from("y"), 0);
        my_node_graph.add_edge(&String::from("o2"), String::from("y"), 0);
        let my_node = Node {
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
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("x"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![
                                (String::from("x"), 0),
                                (String::from("y"), 0),
                            ]),
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
                                id: String::from("2*"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![StreamExpression::SignalCall {
                                id: String::from("y"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                            }],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::from(my_node_graph),
        };

        // node mem(i: int) {
        //     out o: int = 0 fby i;
        // }
        let mut mem_graph = Graph::new();
        mem_graph.add_vertex(String::from("i"), Color::Black);
        mem_graph.add_vertex(String::from("o"), Color::Black);
        mem_graph.add_edge(&String::from("o"), String::from("i"), 1);
        let mem = Node {
            id: String::from("mem"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                Equation {
                    scope: Scope::Output,
                    id: String::from("o"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::SignalCall {
                            id: String::from("i"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i"), 1)]),
                    },
                    location: Location::default(),
                },
            )]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::from(mem_graph),
        };

        // node other_node(v: int) {
        //     out x: int = my_node(y, v).o1;
        //     y: int = 1 + my_node(v-1).o2;
        //     out z: int = mem(z).o;
        // }
        let mut other_node_graph = Graph::new();
        other_node_graph.add_vertex(String::from("v"), Color::Black);
        other_node_graph.add_vertex(String::from("x"), Color::Black);
        other_node_graph.add_vertex(String::from("y"), Color::Black);
        other_node_graph.add_vertex(String::from("z"), Color::Black);
        other_node_graph.add_edge(&String::from("x"), String::from("y"), 0);
        other_node_graph.add_edge(&String::from("x"), String::from("v"), 0);
        other_node_graph.add_edge(&String::from("y"), String::from("v"), 0);
        other_node_graph.add_edge(&String::from("z"), String::from("z"), 1);
        let other_node = Node {
            id: String::from("other_node"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (
                    String::from("x"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("x"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::NodeApplication {
                            node: String::from("my_node"),
                            signal: String::from("o1"),
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("v"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![
                                (String::from("y"), 0),
                                (String::from("v"), 0),
                            ]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("y"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("1+"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![StreamExpression::NodeApplication {
                                node: String::from("my_node"),
                                signal: String::from("o2"),
                                inputs: vec![
                                    StreamExpression::MapApplication {
                                        function_expression: Expression::Call {
                                            id: String::from("-1"),
                                            typing: Some(Type::Abstract(
                                                vec![Type::Integer],
                                                Box::new(Type::Integer),
                                            )),
                                            location: Location::default(),
                                        },
                                        inputs: vec![StreamExpression::SignalCall {
                                            id: String::from("y"),
                                            typing: Type::Integer,
                                            location: Location::default(),
                                            dependencies: Dependencies::from(vec![(
                                                String::from("y"),
                                                0,
                                            )]),
                                        }],
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::from(vec![(
                                            String::from("y"),
                                            0,
                                        )]),
                                    },
                                    StreamExpression::MapApplication {
                                        function_expression: Expression::Call {
                                            id: String::from("-1"),
                                            typing: Some(Type::Abstract(
                                                vec![Type::Integer],
                                                Box::new(Type::Integer),
                                            )),
                                            location: Location::default(),
                                        },
                                        inputs: vec![StreamExpression::SignalCall {
                                            id: String::from("v"),
                                            typing: Type::Integer,
                                            location: Location::default(),
                                            dependencies: Dependencies::from(vec![(
                                                String::from("v"),
                                                0,
                                            )]),
                                        }],
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::from(vec![(
                                            String::from("v"),
                                            0,
                                        )]),
                                    },
                                ],
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                            }],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("z"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("z"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::NodeApplication {
                            node: String::from("mem"),
                            signal: String::from("o"),
                            inputs: vec![StreamExpression::SignalCall {
                                id: String::from("z"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("z"), 0)]),
                            }],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("z"), 1)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::from(other_node_graph),
        };

        let mut file = File {
            typedefs: vec![],
            functions: vec![],
            nodes: vec![mem, my_node, other_node],
            component: None,
            location: Location::default(),
        };

        file.normalize(&mut errors).unwrap();

        // node my_node(x: int, y: int).o1 {
        //     out o1: int = x+y;
        // }
        let mut my_node_o1_graph = Graph::new();
        my_node_o1_graph.add_vertex(String::from("x"), Color::Black);
        my_node_o1_graph.add_vertex(String::from("y"), Color::Black);
        my_node_o1_graph.add_vertex(String::from("o1"), Color::Black);
        my_node_o1_graph.add_edge(&String::from("o1"), String::from("x"), 0);
        my_node_o1_graph.add_edge(&String::from("o1"), String::from("y"), 0);
        let my_node_o1 = UnitaryNode {
            node_id: String::from("my_node"),
            output_id: String::from("o1"),
            inputs: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
            equations: vec![Equation {
                scope: Scope::Output,
                id: String::from("o1"),
                signal_type: Type::Integer,
                expression: StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("+"),
                        typing: Some(Type::Abstract(
                            vec![Type::Integer, Type::Integer],
                            Box::new(Type::Integer),
                        )),
                        location: Location::default(),
                    },
                    inputs: vec![
                        StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                        },
                        StreamExpression::SignalCall {
                            id: String::from("y"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                        },
                    ],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![
                        (String::from("x"), 0),
                        (String::from("y"), 0),
                    ]),
                },
                location: Location::default(),
            }],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::from(my_node_o1_graph),
        };
        // node my_node(y: int).o2 {
        //     out o2: int = 2*y;
        // }
        let mut my_node_o2_graph = Graph::new();
        my_node_o2_graph.add_vertex(String::from("y"), Color::Black);
        my_node_o2_graph.add_vertex(String::from("o2"), Color::Black);
        my_node_o2_graph.add_edge(&String::from("o2"), String::from("y"), 0);
        let my_node_o2 = UnitaryNode {
            node_id: String::from("my_node"),
            output_id: String::from("o2"),
            inputs: vec![(String::from("y"), Type::Integer)],
            equations: vec![Equation {
                scope: Scope::Output,
                id: String::from("o2"),
                signal_type: Type::Integer,
                expression: StreamExpression::MapApplication {
                    function_expression: Expression::Call {
                        id: String::from("2*"),
                        typing: Some(Type::Abstract(
                            vec![Type::Integer, Type::Integer],
                            Box::new(Type::Integer),
                        )),
                        location: Location::default(),
                    },
                    inputs: vec![StreamExpression::SignalCall {
                        id: String::from("y"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                    }],
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                },
                location: Location::default(),
            }],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::from(my_node_o2_graph),
        };
        let mut my_node_graph = Graph::new();
        my_node_graph.add_vertex(String::from("x"), Color::Black);
        my_node_graph.add_vertex(String::from("y"), Color::Black);
        my_node_graph.add_vertex(String::from("o1"), Color::Black);
        my_node_graph.add_vertex(String::from("o2"), Color::Black);
        my_node_graph.add_edge(&String::from("o1"), String::from("x"), 0);
        my_node_graph.add_edge(&String::from("o1"), String::from("y"), 0);
        my_node_graph.add_edge(&String::from("o2"), String::from("y"), 0);
        let my_node = Node {
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
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("x"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![
                                (String::from("x"), 0),
                                (String::from("y"), 0),
                            ]),
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
                                id: String::from("2*"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer, Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![StreamExpression::SignalCall {
                                id: String::from("y"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                            }],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::from([
                (String::from("o1"), my_node_o1),
                (String::from("o2"), my_node_o2),
            ]),
            location: Location::default(),
            graph: OnceCell::from(my_node_graph),
        };

        // node mem(i: int) {
        //     out o: int = 0 fby i;
        // }
        let mut mem_graph = Graph::new();
        mem_graph.add_vertex(String::from("i"), Color::Black);
        mem_graph.add_vertex(String::from("o"), Color::Black);
        mem_graph.add_edge(&String::from("o"), String::from("i"), 1);
        let mem_unitary = UnitaryNode {
            node_id: String::from("mem"),
            output_id: String::from("o"),
            inputs: vec![(String::from("i"), Type::Integer)],
            equations: vec![Equation {
                scope: Scope::Output,
                id: String::from("o"),
                signal_type: Type::Integer,
                expression: StreamExpression::FollowedBy {
                    constant: Constant::Integer(0),
                    expression: Box::new(StreamExpression::SignalCall {
                        id: String::from("i"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                    }),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("i"), 1)]),
                },
                location: Location::default(),
            }],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::from(mem_graph.clone()),
        };
        let mem = Node {
            id: String::from("mem"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([(
                String::from("o"),
                Equation {
                    scope: Scope::Output,
                    id: String::from("o"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::FollowedBy {
                        constant: Constant::Integer(0),
                        expression: Box::new(StreamExpression::SignalCall {
                            id: String::from("i"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                        }),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i"), 1)]),
                    },
                    location: Location::default(),
                },
            )]),
            unitary_nodes: HashMap::from([(String::from("o"), mem_unitary)]),
            location: Location::default(),
            graph: OnceCell::from(mem_graph),
        };

        // node other_node(v: int).x {
        //     x_1: int = v-1;
        //     x_2: int = my_node(x_1).o2;
        //     y: int = 1 + x_2;
        //     out x: int = my_node(y, v).o1;
        // }
        let mut other_node_x_graph = Graph::new();
        other_node_x_graph.add_vertex(String::from("v"), Color::Black);
        other_node_x_graph.add_vertex(String::from("y"), Color::Black);
        other_node_x_graph.add_vertex(String::from("x"), Color::Black);
        other_node_x_graph.add_edge(&String::from("y"), String::from("v"), 0);
        other_node_x_graph.add_edge(&String::from("x"), String::from("y"), 0);
        other_node_x_graph.add_edge(&String::from("x"), String::from("v"), 0);
        let other_node_x = UnitaryNode {
            node_id: String::from("other_node"),
            output_id: String::from("x"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![
                Equation {
                    scope: Scope::Local,
                    id: String::from("x_1"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("-1"),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            id: String::from("v"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                    },
                    location: Location::default(),
                },
                Equation {
                    scope: Scope::Local,
                    id: String::from("x_2"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::UnitaryNodeApplication {
                        id: Some(format!("my_nodeo2x_2")),
                        node: String::from("my_node"),
                        signal: String::from("o2"),
                        inputs: vec![(
                            format!("y"),
                            StreamExpression::SignalCall {
                                id: String::from("x_1"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("x_1"), 0)]),
                            },
                        )],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x_1"), 0)]),
                    },
                    location: Location::default(),
                },
                Equation {
                    scope: Scope::Local,
                    id: String::from("y"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::MapApplication {
                        function_expression: Expression::Call {
                            id: String::from("1+"),
                            typing: Some(Type::Abstract(
                                vec![Type::Integer],
                                Box::new(Type::Integer),
                            )),
                            location: Location::default(),
                        },
                        inputs: vec![StreamExpression::SignalCall {
                            id: String::from("x_2"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x_2"), 0)]),
                        }],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("x_2"), 0)]),
                    },
                    location: Location::default(),
                },
                Equation {
                    scope: Scope::Output,
                    id: String::from("x"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::UnitaryNodeApplication {
                        id: Some(format!("my_nodeo1x")),
                        node: String::from("my_node"),
                        signal: String::from("o1"),
                        inputs: vec![
                            (
                                format!("x"),
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                            ),
                            (
                                format!("y"),
                                StreamExpression::SignalCall {
                                    id: String::from("v"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                                },
                            ),
                        ],
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![
                            (String::from("y"), 0),
                            (String::from("v"), 0),
                        ]),
                    },
                    location: Location::default(),
                },
            ],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::from(other_node_x_graph),
        };
        // node other_node().z {
        //     out z: int = 0 fby z;
        // }
        let mut other_node_z_graph = Graph::new();
        other_node_z_graph.add_vertex(String::from("z"), Color::Black);
        other_node_z_graph.add_edge(&String::from("z"), String::from("z"), 1);
        let other_node_z = UnitaryNode {
            node_id: String::from("other_node"),
            output_id: String::from("z"),
            inputs: vec![],
            equations: vec![Equation {
                scope: Scope::Output,
                id: String::from("z"),
                signal_type: Type::Integer,
                expression: StreamExpression::FollowedBy {
                    constant: Constant::Integer(0),
                    expression: Box::new(StreamExpression::SignalCall {
                        id: String::from("z"),
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("z"), 0)]),
                    }),
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("z"), 1)]),
                },
                location: Location::default(),
            }],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::from(other_node_z_graph),
        };
        let mut other_node_graph = Graph::new();
        other_node_graph.add_vertex(String::from("v"), Color::Black);
        other_node_graph.add_vertex(String::from("x"), Color::Black);
        other_node_graph.add_vertex(String::from("y"), Color::Black);
        other_node_graph.add_vertex(String::from("z"), Color::Black);
        other_node_graph.add_edge(&String::from("x"), String::from("y"), 0);
        other_node_graph.add_edge(&String::from("x"), String::from("v"), 0);
        other_node_graph.add_edge(&String::from("y"), String::from("v"), 0);
        other_node_graph.add_edge(&String::from("z"), String::from("z"), 1);
        let other_node = Node {
            id: String::from("other_node"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (
                    String::from("x"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("x"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::NodeApplication {
                            node: String::from("my_node"),
                            signal: String::from("o1"),
                            inputs: vec![
                                StreamExpression::SignalCall {
                                    id: String::from("y"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
                                },
                                StreamExpression::SignalCall {
                                    id: String::from("v"),
                                    typing: Type::Integer,
                                    location: Location::default(),
                                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                                },
                            ],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![
                                (String::from("y"), 0),
                                (String::from("v"), 0),
                            ]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("y"),
                    Equation {
                        scope: Scope::Local,
                        id: String::from("y"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::MapApplication {
                            function_expression: Expression::Call {
                                id: String::from("1+"),
                                typing: Some(Type::Abstract(
                                    vec![Type::Integer],
                                    Box::new(Type::Integer),
                                )),
                                location: Location::default(),
                            },
                            inputs: vec![StreamExpression::NodeApplication {
                                node: String::from("my_node"),
                                signal: String::from("o2"),
                                inputs: vec![
                                    StreamExpression::MapApplication {
                                        function_expression: Expression::Call {
                                            id: String::from("-1"),
                                            typing: Some(Type::Abstract(
                                                vec![Type::Integer],
                                                Box::new(Type::Integer),
                                            )),
                                            location: Location::default(),
                                        },
                                        inputs: vec![StreamExpression::SignalCall {
                                            id: String::from("y"),
                                            typing: Type::Integer,
                                            location: Location::default(),
                                            dependencies: Dependencies::from(vec![(
                                                String::from("y"),
                                                0,
                                            )]),
                                        }],
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::from(vec![(
                                            String::from("y"),
                                            0,
                                        )]),
                                    },
                                    StreamExpression::MapApplication {
                                        function_expression: Expression::Call {
                                            id: String::from("-1"),
                                            typing: Some(Type::Abstract(
                                                vec![Type::Integer],
                                                Box::new(Type::Integer),
                                            )),
                                            location: Location::default(),
                                        },
                                        inputs: vec![StreamExpression::SignalCall {
                                            id: String::from("v"),
                                            typing: Type::Integer,
                                            location: Location::default(),
                                            dependencies: Dependencies::from(vec![(
                                                String::from("v"),
                                                0,
                                            )]),
                                        }],
                                        typing: Type::Integer,
                                        location: Location::default(),
                                        dependencies: Dependencies::from(vec![(
                                            String::from("v"),
                                            0,
                                        )]),
                                    },
                                ],
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                            }],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
                (
                    String::from("z"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("z"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::NodeApplication {
                            node: String::from("mem"),
                            signal: String::from("o"),
                            inputs: vec![StreamExpression::SignalCall {
                                id: String::from("z"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("z"), 0)]),
                            }],
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("z"), 1)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::from([
                (String::from("x"), other_node_x),
                (String::from("z"), other_node_z),
            ]),
            location: Location::default(),
            graph: OnceCell::from(other_node_graph),
        };

        let control = File {
            typedefs: vec![],
            functions: vec![],
            nodes: vec![mem, my_node, other_node],
            component: None,
            location: Location::default(),
        };

        assert_eq!(file, control);
    }
}
