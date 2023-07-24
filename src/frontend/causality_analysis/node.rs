use crate::{error::Error, hir::node::Node};

impl Node {
    /// Check the causality of the node.
    ///
    /// # Example
    /// The folowing simple node is causal, there is no causality loop.
    /// ```GR
    /// node causal_node1(i: int) {
    ///     out o: int = x;
    ///     x: int = i;
    /// }
    /// ```
    ///
    /// The next node is causal as well, `x` does not depends on `o` but depends
    /// on the memory of `o`. Then there is no causality loop.
    /// ```GR
    /// node causal_node2() {
    ///     out o: int = x;
    ///     x: int = 0 fby o;
    /// }
    /// ```
    ///
    /// But the node that follows is not causal, `o` depends on `x` which depends
    /// on `o`. Values of signals can not be determined, then the compilation
    /// raises a causality error.
    /// ```GR
    /// node not_causal_node(i: int) {
    ///     out o: int = x;
    ///     x: int = o;
    /// }
    /// ```
    pub fn causal(&self, errors: &mut Vec<Error>) -> Result<(), ()> {
        // construct node's subgraph containing only 0-depth dependencies
        let mut subgraph = self
            .graph
            .get()
            .unwrap()
            .subgraph_on_edges(|weight| weight == 0);

        // if a schedule exists, then the node is causal
        let _ = subgraph.topological_sorting(errors).map_err(|signal| {
            let error = Error::NotCausal {
                node: self.id.clone(),
                signal: signal,
                location: self.location.clone(),
            };
            errors.push(error)
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod causal {
    use once_cell::sync::OnceCell;
    use std::collections::HashMap;

    use crate::common::{
        constant::Constant,
        graph::{color::Color, Graph},
        location::Location,
        r#type::Type,
        scope::Scope,
    };
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, node::Node,
        stream_expression::StreamExpression,
    };

    #[test]
    fn should_accept_causal_nodes() {
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

        let mut errors = vec![];
        node.causal(&mut errors).unwrap();
    }

    #[test]
    fn should_accept_shifted_causality_loops() {
        let node = Node {
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
        node.graph.set(graph).unwrap();

        let mut errors = vec![];
        node.causal(&mut errors).unwrap();
    }

    #[test]
    fn should_not_accept_direct_causality_loops() {
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
                            id: String::from("o"),
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
        graph.add_edge(&String::from("x"), String::from("o"), 0);
        graph.add_edge(&String::from("o"), String::from("x"), 0);
        node.graph.set(graph).unwrap();

        let mut errors = vec![];
        node.causal(&mut errors).unwrap_err();
    }
}
