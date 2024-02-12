use petgraph::algo::toposort;

use crate::{
    common::graph::neighbor::Label,
    error::{Error, TerminationError},
    hir::node::Node,
};

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
    pub fn causal(&self, errors: &mut Vec<Error>) -> Result<(), TerminationError> {
        // construct node's subgraph containing only 0-depth
        let graph = self
            .graph
            .get()
            .expect("node dependency graph should be computed");
        let mut subgraph = graph.clone();
        graph.all_edges().for_each(|(from, to, label)| match label {
            Label::Weight(0) => (),
            _ => assert_ne!(subgraph.remove_edge(from, to), Some(Label::Weight(0))),
        });

        // if a schedule exists, then the node is causal
        let _ = toposort(&subgraph, None).map_err(|signal| {
            let error = Error::NotCausal {
                node: self.id.clone(),
                signal: signal.node_id().clone(),
                location: self.location.clone(),
            };
            errors.push(error);
            TerminationError
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod causal {
    use std::collections::HashMap;

    use petgraph::graphmap::GraphMap;

    use crate::common::{
        constant::Constant, graph::neighbor::Label, location::Location, r#type::Type, scope::Scope,
    };
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, node::Node, once_cell::OnceCell,
        signal::Signal, stream_expression::StreamExpression,
    };

    #[test]
    fn should_accept_causal_nodes() {
        let node = Node {
            contract: Default::default(),
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
                            signal: Signal {
                                id: String::from("x"),
                                scope: Scope::Local,
                            },
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
                            signal: Signal {
                                id: String::from("i"),
                                scope: Scope::Input,
                            },
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

        let mut graph = GraphMap::new();
        graph.add_node(String::from("o"));
        graph.add_node(String::from("x"));
        graph.add_node(String::from("i"));
        graph.add_edge(String::from("x"), String::from("i"), Label::Weight(0));
        graph.add_edge(String::from("o"), String::from("x"), Label::Weight(0));
        node.graph.set(graph).unwrap();

        let mut errors = vec![];
        node.causal(&mut errors).unwrap();
    }

    #[test]
    fn should_accept_shifted_causality_loops() {
        let node = Node {
            contract: Default::default(),
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
                            signal: Signal {
                                id: String::from("x"),
                                scope: Scope::Local,
                            },
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
                                signal: Signal {
                                    id: String::from("o"),
                                    scope: Scope::Output,
                                },
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

        let mut graph = GraphMap::new();
        graph.add_node(String::from("o"));
        graph.add_node(String::from("x"));
        graph.add_edge(String::from("x"), String::from("o"), Label::Weight(1));
        graph.add_edge(String::from("o"), String::from("x"), Label::Weight(0));
        node.graph.set(graph).unwrap();

        let mut errors = vec![];
        node.causal(&mut errors).unwrap();
    }

    #[test]
    fn should_not_accept_direct_causality_loops() {
        let node = Node {
            contract: Default::default(),
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
                            signal: Signal {
                                id: String::from("x"),
                                scope: Scope::Local,
                            },
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
                            signal: Signal {
                                id: String::from("o"),
                                scope: Scope::Output,
                            },
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

        let mut graph = GraphMap::new();
        graph.add_node(String::from("o"));
        graph.add_node(String::from("x"));
        graph.add_node(String::from("i"));
        graph.add_edge(String::from("x"), String::from("o"), Label::Weight(0));
        graph.add_edge(String::from("o"), String::from("x"), Label::Weight(0));
        node.graph.set(graph).unwrap();

        let mut errors = vec![];
        node.causal(&mut errors).unwrap_err();
    }
}
