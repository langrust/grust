use crate::{error::Error, hir::file::File};

impl File {
    /// Check the causality of the file.
    ///
    /// # Example
    /// The folowing file is causal, there is no causality loop.
    /// ```GR
    /// node causal_node(i: int) {
    ///     out o: int = x;
    ///     x: int = i;
    /// }
    ///
    /// component causal_component() {
    ///     out o: int = x;
    ///     x: int = 0 fby o;
    /// }
    /// ```
    ///
    /// But the file that follows is not causal.
    /// In the node `not_causal_node`, signal`o` depends on `x` which depends
    /// on `o`. Values of signals can not be determined, then the compilation
    /// raises a causality error.
    /// ```GR
    /// node not_causal_node(i: int) {
    ///     out o: int = x;
    ///     x: int = o;
    /// }
    ///
    /// component causal_component() {
    ///     out o: int = x;
    ///     x: int = 0 fby o;
    /// }
    /// ```
    pub fn causality_analysis(&self, errors: &mut Vec<Error>) -> Result<(), ()> {
        // check causality for each node
        self.nodes
            .iter()
            .map(|node| node.causal(errors))
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Result<_, _>>()?;
        // check causality of the optional component
        self.component
            .as_ref()
            .map_or(Ok(()), |component| component.causal(errors))
    }
}

#[cfg(test)]
mod causality_analysis {
    use once_cell::sync::OnceCell;
    use std::collections::HashMap;

    use crate::hir::{
        dependencies::Dependencies, equation::Equation, node::Node,
        stream_expression::StreamExpression,
    };
    use crate::{
        common::{
            constant::Constant,
            graph::{color::Color, Graph},
            location::Location,
            r#type::Type,
            scope::Scope,
        },
        hir::file::File,
    };

    #[test]
    fn should_accept_causal_files() {
        let node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
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
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("i"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("i"), 0);
        graph.add_edge(&String::from("o"), String::from("x"), 0);
        node.graph.set(graph).unwrap();

        let component = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
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
                        expression: StreamExpression::FollowedBy {
                            constant: Constant::Integer(0),
                            expression: Box::new(StreamExpression::SignalCall {
                                id: String::from("o"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                            }),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("o"), 1)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("o"), 1);
        graph.add_edge(&String::from("o"), String::from("x"), 0);
        component.graph.set(graph).unwrap();

        let file = File {
            typedefs: vec![],
            functions: vec![],
            nodes: vec![node],
            component: Some(component),
            location: Location::default(),
        };

        let mut errors = vec![];
        file.causality_analysis(&mut errors).unwrap();
    }

    #[test]
    fn should_raise_error_when_one_node_is_not_causal() {
        let node1 = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
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
                            id: String::from("o"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("o"), 0);
        graph.add_edge(&String::from("o"), String::from("x"), 0);
        node1.graph.set(graph).unwrap();

        let node2 = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
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
                        expression: StreamExpression::FollowedBy {
                            constant: Constant::Integer(0),
                            expression: Box::new(StreamExpression::SignalCall {
                                id: String::from("o"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                            }),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("o"), 1)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("o"), 1);
        graph.add_edge(&String::from("o"), String::from("x"), 0);
        node2.graph.set(graph).unwrap();

        let file = File {
            typedefs: vec![],
            functions: vec![],
            nodes: vec![node1, node2],
            component: None,
            location: Location::default(),
        };

        let mut errors = vec![];
        file.causality_analysis(&mut errors).unwrap_err();
    }

    #[test]
    fn should_raise_error_when_many_nodes_are_not_causal() {
        let node1 = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
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
                            id: String::from("o"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("o"), 0);
        graph.add_edge(&String::from("o"), String::from("x"), 0);
        node1.graph.set(graph).unwrap();

        let node2 = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("o"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
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
                        expression: StreamExpression::FollowedBy {
                            constant: Constant::Integer(0),
                            expression: Box::new(StreamExpression::SignalCall {
                                id: String::from("o"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                            }),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("o"), 1)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("o"), 1);
        graph.add_edge(&String::from("o"), String::from("o"), 0);
        node2.graph.set(graph).unwrap();

        let file = File {
            typedefs: vec![],
            functions: vec![],
            nodes: vec![node1, node2],
            component: None,
            location: Location::default(),
        };

        let mut errors = vec![];
        file.causality_analysis(&mut errors).unwrap_err();
    }

    #[test]
    fn should_raise_error_when_the_component_is_not_causal() {
        let node = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("i"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("x"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("x"), 0)]),
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
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_vertex(String::from("i"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("i"), 0);
        graph.add_edge(&String::from("o"), String::from("x"), 0);
        node.graph.set(graph).unwrap();

        let component = Node {
            id: String::from("test"),
            is_component: false,
            inputs: vec![],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            id: String::from("o"),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
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
                        expression: StreamExpression::FollowedBy {
                            constant: Constant::Integer(0),
                            expression: Box::new(StreamExpression::SignalCall {
                                id: String::from("o"),
                                typing: Type::Integer,
                                location: Location::default(),
                                dependencies: Dependencies::from(vec![(String::from("o"), 0)]),
                            }),
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("o"), 1)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = Graph::new();
        graph.add_vertex(String::from("o"), Color::Black);
        graph.add_vertex(String::from("x"), Color::Black);
        graph.add_edge(&String::from("x"), String::from("o"), 1);
        graph.add_edge(&String::from("o"), String::from("o"), 0);
        component.graph.set(graph).unwrap();

        let file = File {
            typedefs: vec![],
            functions: vec![],
            nodes: vec![node],
            component: Some(component),
            location: Location::default(),
        };

        let mut errors = vec![];
        file.causality_analysis(&mut errors).unwrap_err();
    }
}
