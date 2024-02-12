use std::collections::HashMap;

use petgraph::algo::has_path_connecting;
use petgraph::graphmap::DiGraphMap;

use crate::common::graph::neighbor::Label;
use crate::error::{Error, TerminationError};
use crate::hir::contract::Term;
use crate::hir::{memory::Memory, node::Node, once_cell::OnceCell, unitary_node::UnitaryNode};
use crate::{common::scope::Scope, hir::contract::Contract};

pub type UsedInputs = Vec<(String, bool)>;

impl Node {
    /// Add inputs that are used by the unitary node.
    pub fn add_used_inputs(&self, used_inputs: &mut HashMap<(String, String), UsedInputs>) {
        self.unitary_nodes
            .iter()
            .for_each(|(output, unitary_node)| {
                assert!(used_inputs
                    .insert(
                        (self.id.clone(), output.clone()),
                        self.inputs
                            .iter()
                            .map(|input| { (input.0.clone(), unitary_node.inputs.contains(input)) })
                            .collect::<Vec<_>>(),
                    )
                    .is_none());
            })
    }

    /// Change every node application into unitary node application.
    ///
    /// It removes unused inputs from unitary node application.
    ///
    /// # Example
    ///
    /// Let be a node `my_node` as follows:
    ///
    /// ```GR
    /// node my_node(x: int, y: int) {
    ///     out o1: int = x+y;
    ///     out o2: int = 2*y;
    /// }
    /// ```
    ///
    /// The application of the node `my_node(g-1, v).o2` is changed
    /// to the application of the unitary node `my_node(v).o2`
    pub fn change_node_application_into_unitary_node_application(
        &mut self,
        used_inputs: &HashMap<(String, String), UsedInputs>,
    ) {
        self.unitary_nodes.values_mut().for_each(|unitary_node| {
            unitary_node.equations.iter_mut().for_each(|equation| {
                equation
                    .expression
                    .change_node_application_into_unitary_node_application(used_inputs)
            })
        })
    }

    /// Generate unitary nodes from mother node.
    ///
    /// Generate and add unitary nodes to mother node.
    /// Unitary nodes are nodes with one output and contains
    /// all signals from which the output computation depends.
    ///
    /// It also detects unused signal definitions or inputs.
    pub fn generate_unitary_nodes(
        &mut self,
        creusot_contract: bool,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        // get outputs identifiers
        let outputs = self
            .unscheduled_equations
            .values()
            .filter(|equation| equation.scope.eq(&Scope::Output))
            .map(|equation| equation.id.clone())
            .collect::<Vec<_>>();

        // construct unitary node for each output and get used signals
        let used_signals = outputs
            .into_iter()
            .map(|output| self.add_unitary_node(output, creusot_contract))
            .flat_map(|subgraph| subgraph.nodes())
            .collect::<Vec<_>>();

        // check that every signals are used
        let graph = self
            .graph
            .get()
            .expect("node dependency graph should be computed");
        let unused_signals = graph
            .nodes()
            .filter(|id| used_signals.contains(id))
            .collect::<Vec<_>>();
        unused_signals
            .into_iter()
            .map(|signal| {
                let error = Error::UnusedSignal {
                    node: self.id.clone(),
                    signal: signal.clone(),
                    location: self.location.clone(),
                };
                errors.push(error);
                Err(TerminationError)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Result<_, _>>()
    }

    fn add_unitary_node(
        &mut self,
        output: String,
        creusot_contract: bool,
    ) -> DiGraphMap<String, Label> {
        let Node {
            contract:
                Contract {
                    requires,
                    ensures,
                    invariant,
                },
            id: node,
            inputs,
            unscheduled_equations,
            unitary_nodes,
            location,
            ..
        } = self;

        // construct unitary node's subgraph from its output
        let graph = self
            .graph
            .get()
            .expect("node dependency graph should be computed");
        let mut subgraph = graph.clone();
        graph.nodes().for_each(|id| {
            let has_path = has_path_connecting(graph, id, &output, None); // TODO: contrary?
            if !has_path {
                subgraph.remove_node(id);
            }
        });

        // get useful inputs (in application order)
        let unitary_node_inputs = inputs
            .iter_mut()
            .filter(|(id, _)| subgraph.contains_node(id))
            .map(|input| input.clone())
            .collect::<Vec<_>>();

        // retrieve equations from useful signals
        let equations = subgraph
            .nodes()
            .filter_map(|signal| unscheduled_equations.get(signal))
            .cloned()
            .collect();

        // retrieve contract from usefull signals
        let retrieve_terms = |terms: &Vec<Term>| {
            terms
                .iter()
                .filter_map(|term| {
                    if subgraph.nodes().any(|signal| term.contains_id(signal)) {
                        Some(term)
                    } else {
                        None
                    }
                })
                .cloned()
                .collect::<Vec<_>>()
        };
        let contract = Contract {
            requires: retrieve_terms(requires),
            ensures: retrieve_terms(ensures),
            invariant: retrieve_terms(invariant),
        };

        // construct unitary node
        let unitary_node = UnitaryNode {
            contract,
            node_id: node.clone(),
            output_id: output.clone(),
            inputs: unitary_node_inputs,
            equations,
            memory: Memory::new(),
            location: location.clone(),
            graph: OnceCell::new(),
        };

        // insert it in node's storage
        unitary_nodes.insert(output, unitary_node);

        subgraph
    }
}

#[cfg(test)]
mod add_unitary_node {
    use std::collections::HashMap;

    use petgraph::graphmap::GraphMap;

    use crate::common::graph::neighbor::Label;
    use crate::hir::{
        equation::Equation, memory::Memory, node::Node, once_cell::OnceCell, signal::Signal,
        stream_expression::StreamExpression, unitary_node::UnitaryNode,
    };
    use crate::{
        common::{location::Location, r#type::Type, scope::Scope},
        hir::dependencies::Dependencies,
    };

    #[test]
    fn should_add_unitary_node_computing_output() {
        let mut node = Node {
            contract: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
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
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("x_2"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                                id: String::from("x_1"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
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
        graph.add_node(String::from("i1"));
        graph.add_node(String::from("i2"));
        graph.add_node(String::from("x"));
        graph.add_node(String::from("o1"));
        graph.add_node(String::from("o2"));
        graph.add_edge(String::from("x"), String::from("i1"), Label::Weight(0));
        graph.add_edge(String::from("o1"), String::from("x"), Label::Weight(0));
        graph.add_edge(String::from("o2"), String::from("i2"), Label::Weight(0));
        node.graph.set(graph).unwrap();

        node.add_unitary_node(String::from("o1"), false);

        let unitary_node = UnitaryNode {
            contract: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("o1"),
            inputs: vec![(String::from("i1"), Type::Integer)],
            equations: vec![
                Equation {
                    scope: Scope::Output,
                    id: String::from("o1"),
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
                Equation {
                    scope: Scope::Local,
                    id: String::from("x"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x_1"),
                            scope: Scope::Input,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                    },
                    location: Location::default(),
                },
            ],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let control = Node {
            contract: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
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
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("x_2"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                                id: String::from("x_1"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::from([(String::from("o1"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = GraphMap::new();
        graph.add_node(String::from("i1"));
        graph.add_node(String::from("i2"));
        graph.add_node(String::from("x"));
        graph.add_node(String::from("o1"));
        graph.add_node(String::from("o2"));
        graph.add_edge(String::from("x"), String::from("i1"), Label::Weight(0));
        graph.add_edge(String::from("o1"), String::from("x"), Label::Weight(0));
        graph.add_edge(String::from("o2"), String::from("i2"), Label::Weight(0));
        control.graph.set(graph.clone()).unwrap();

        assert!(node.eq_unscheduled(&control))
    }
}

#[cfg(test)]
mod generate_unitary_nodes {

    use petgraph::graphmap::GraphMap;

    use crate::common::graph::neighbor::Label;
    use crate::hir::{
        equation::Equation, memory::Memory, node::Node, once_cell::OnceCell, signal::Signal,
        stream_expression::StreamExpression, unitary_node::UnitaryNode,
    };
    use crate::{
        common::{location::Location, r#type::Type, scope::Scope},
        hir::dependencies::Dependencies,
    };
    use std::collections::HashMap;

    #[test]
    fn should_generate_unitary_nodes_as_expected() {
        let mut errors = vec![];

        let mut node = Node {
            contract: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
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
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("x_2"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                                id: String::from("x_1"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
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
        graph.add_node(String::from("i1"));
        graph.add_node(String::from("i2"));
        graph.add_node(String::from("x"));
        graph.add_node(String::from("o1"));
        graph.add_node(String::from("o2"));
        graph.add_edge(String::from("x"), String::from("i1"), Label::Weight(0));
        graph.add_edge(String::from("o1"), String::from("x"), Label::Weight(0));
        graph.add_edge(String::from("o2"), String::from("i2"), Label::Weight(0));
        node.graph.set(graph).unwrap();

        node.generate_unitary_nodes(false, &mut errors).unwrap();

        let unitary_node_1 = UnitaryNode {
            contract: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("o1"),
            inputs: vec![(String::from("i1"), Type::Integer)],
            equations: vec![
                Equation {
                    scope: Scope::Output,
                    id: String::from("o1"),
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
                Equation {
                    scope: Scope::Local,
                    id: String::from("x"),
                    signal_type: Type::Integer,
                    expression: StreamExpression::SignalCall {
                        signal: Signal {
                            id: String::from("x_1"),
                            scope: Scope::Input,
                        },
                        typing: Type::Integer,
                        location: Location::default(),
                        dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                    },
                    location: Location::default(),
                },
            ],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let unitary_node_2 = UnitaryNode {
            contract: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("o2"),
            inputs: vec![(String::from("i2"), Type::Integer)],
            equations: vec![Equation {
                scope: Scope::Output,
                id: String::from("o2"),
                signal_type: Type::Integer,
                expression: StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("x_2"),
                        scope: Scope::Input,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
                },
                location: Location::default(),
            }],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let control = Node {
            contract: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
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
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("x_2"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                                id: String::from("x_1"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
                        },
                        location: Location::default(),
                    },
                ),
            ]),
            unitary_nodes: HashMap::from([
                (String::from("o2"), unitary_node_2),
                (String::from("o1"), unitary_node_1),
            ]),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let mut graph = GraphMap::new();
        graph.add_node(String::from("i1"));
        graph.add_node(String::from("i2"));
        graph.add_node(String::from("x"));
        graph.add_node(String::from("o1"));
        graph.add_node(String::from("o2"));
        graph.add_edge(String::from("x"), String::from("i1"), Label::Weight(0));
        graph.add_edge(String::from("o1"), String::from("x"), Label::Weight(0));
        graph.add_edge(String::from("o2"), String::from("i2"), Label::Weight(0));
        control.graph.set(graph.clone()).unwrap();

        assert!(node.eq_unscheduled(&control));
    }

    #[test]
    fn should_generate_unitary_nodes_for_every_output() {
        let mut errors = vec![];

        let mut node = Node {
            contract: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
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
                    String::from("o2"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o2"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("x_2"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i2"), 0)]),
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
                                id: String::from("x_1"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
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
        graph.add_node(String::from("i1"));
        graph.add_node(String::from("i2"));
        graph.add_node(String::from("x"));
        graph.add_node(String::from("o1"));
        graph.add_node(String::from("o2"));
        graph.add_edge(String::from("x"), String::from("i1"), Label::Weight(0));
        graph.add_edge(String::from("o1"), String::from("x"), Label::Weight(0));
        graph.add_edge(String::from("o2"), String::from("i2"), Label::Weight(0));
        node.graph.set(graph).unwrap();

        node.generate_unitary_nodes(false, &mut errors).unwrap();

        let mut output_equations = node
            .unscheduled_equations
            .iter()
            .filter(|(_, equation)| equation.scope.eq(&Scope::Output));

        assert!(output_equations.all(|(signal, _)| node.unitary_nodes.contains_key(signal)))
    }

    #[test]
    fn should_raise_error_for_unused_signals() {
        let mut errors = vec![];

        let mut node = Node {
            contract: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![
                (String::from("i1"), Type::Integer),
                (String::from("i2"), Type::Integer),
            ],
            unscheduled_equations: HashMap::from([
                (
                    String::from("o1"),
                    Equation {
                        scope: Scope::Output,
                        id: String::from("o1"),
                        signal_type: Type::Integer,
                        expression: StreamExpression::SignalCall {
                            signal: Signal {
                                id: String::from("x_1"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
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
                                id: String::from("x_1"),
                                scope: Scope::Input,
                            },
                            typing: Type::Integer,
                            location: Location::default(),
                            dependencies: Dependencies::from(vec![(String::from("i1"), 0)]),
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
        graph.add_node(String::from("i1"));
        graph.add_node(String::from("i2"));
        graph.add_node(String::from("x"));
        graph.add_node(String::from("o1"));
        graph.add_edge(String::from("x"), String::from("i1"), Label::Weight(0));
        graph.add_edge(String::from("o1"), String::from("i1"), Label::Weight(0));
        node.graph.set(graph).unwrap();

        node.generate_unitary_nodes(false, &mut errors).unwrap_err();
    }
}
