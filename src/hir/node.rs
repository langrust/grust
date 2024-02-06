use std::collections::HashMap;

use crate::ast::term::Contract;
use crate::common::{
    graph::{color::Color, Graph},
    location::Location,
    r#type::Type,
    serialize::ordered_map,
};
use crate::hir::{equation::Equation, once_cell::OnceCell, unitary_node::UnitaryNode};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust node HIR.
pub struct Node {
    /// Node identifier.
    pub id: String,
    /// Is true when the node is a component.
    pub is_component: bool,
    /// Node's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Node's unscheduled equations.    
    #[serde(serialize_with = "ordered_map")]
    pub unscheduled_equations: HashMap<String, Equation>,
    /// Unitary output nodes generated from this node.
    #[serde(serialize_with = "ordered_map")]
    pub unitary_nodes: HashMap<String, UnitaryNode>,
    /// Node's contracts.
    pub contracts: Contract,
    /// Node location.
    pub location: Location,
    /// Node dependency graph.
    pub graph: OnceCell<Graph<Color>>,
}

impl Node {
    /// Tells if two unscheduled nodes are equal.
    pub fn eq_unscheduled(&self, other: &Node) -> bool {
        self.id == other.id
            && self.is_component == other.is_component
            && self.inputs == other.inputs
            && self.unscheduled_equations == other.unscheduled_equations
            && self.unitary_nodes.len() == other.unitary_nodes.len()
            && self.unitary_nodes.iter().all(|(output_id, unitary_node)| {
                other
                    .unitary_nodes
                    .get(output_id)
                    .map_or(false, |other_unitary_node| {
                        unitary_node.eq_unscheduled(other_unitary_node)
                    })
            })
            && self.location == other.location
            && self.graph == other.graph
    }
}

#[cfg(test)]
mod eq_unscheduled {
    use std::collections::HashMap;

    use crate::common::{constant::Constant, location::Location, r#type::Type, scope::Scope};
    use crate::hir::{
        dependencies::Dependencies, equation::Equation, memory::Memory, node::Node,
        once_cell::OnceCell, signal::Signal, stream_expression::StreamExpression,
        unitary_node::UnitaryNode,
    };

    #[test]
    fn should_return_true_for_strictly_equal_nodes() {
        let equation = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("y"),
                    scope: Scope::Input,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
            },
            location: Location::default(),
        };
        let unitary_node = UnitaryNode {
            contracts: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![(String::from("y"), Type::Integer)],
            equations: vec![equation.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let node = Node {
            contracts: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("y"), Type::Integer)],
            unscheduled_equations: HashMap::from([(String::from("x"), equation)]),
            unitary_nodes: HashMap::from([(String::from("x"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let other = node.clone();

        assert!(node.eq_unscheduled(&other))
    }

    #[test]
    fn should_return_true_for_nodes_with_unscheduled_unitary_nodes() {
        let equation_1 = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("y"),
                    scope: Scope::Local,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
            },
            location: Location::default(),
        };
        let equation_2 = Equation {
            scope: Scope::Local,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
                constant: Constant::Integer(0),
                expression: Box::new(StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("v"),
                        scope: Scope::Input,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                }),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("v"), 1)]),
            },
            location: Location::default(),
        };
        let unitary_node = UnitaryNode {
            contracts: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_1.clone(), equation_2.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let node = Node {
            contracts: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1.clone()),
                (String::from("y"), equation_2.clone()),
            ]),
            unitary_nodes: HashMap::from([(String::from("x"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let unitary_node = UnitaryNode {
            contracts: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_2.clone(), equation_1.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let other = Node {
            contracts: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1),
                (String::from("y"), equation_2),
            ]),
            unitary_nodes: HashMap::from([(String::from("x"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        assert!(node.eq_unscheduled(&other))
    }

    #[test]
    fn should_return_false_for_unequal_unitary_nodes() {
        let equation_1 = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("y"),
                    scope: Scope::Local,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
            },
            location: Location::default(),
        };
        let equation_2 = Equation {
            scope: Scope::Local,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
                constant: Constant::Integer(0),
                expression: Box::new(StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("v"),
                        scope: Scope::Input,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                }),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("v"), 1)]),
            },
            location: Location::default(),
        };
        let unitary_node = UnitaryNode {
            contracts: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_1.clone(), equation_2.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let node = Node {
            contracts: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1.clone()),
                (String::from("y"), equation_2.clone()),
            ]),
            unitary_nodes: HashMap::from([(String::from("x"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let unitary_node = UnitaryNode {
            contracts: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_2.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let other = Node {
            contracts: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1),
                (String::from("y"), equation_2),
            ]),
            unitary_nodes: HashMap::from([(String::from("x"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        assert!(!node.eq_unscheduled(&other))
    }

    #[test]
    fn should_return_false_for_missing_unitary_nodes() {
        let equation_1 = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("y"),
                    scope: Scope::Local,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
            },
            location: Location::default(),
        };
        let equation_2 = Equation {
            scope: Scope::Local,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
                constant: Constant::Integer(0),
                expression: Box::new(StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("v"),
                        scope: Scope::Input,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                }),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("v"), 1)]),
            },
            location: Location::default(),
        };
        let unitary_node_1 = UnitaryNode {
            contracts: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_1.clone(), equation_2.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let unitary_node_2 = UnitaryNode {
            contracts: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("y"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_2.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let node = Node {
            contracts: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1.clone()),
                (String::from("y"), equation_2.clone()),
            ]),
            unitary_nodes: HashMap::from([
                (String::from("x"), unitary_node_1.clone()),
                (String::from("y"), unitary_node_2.clone()),
            ]),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let other = Node {
            contracts: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1),
                (String::from("y"), equation_2),
            ]),
            unitary_nodes: HashMap::from([
                (String::from("x"), unitary_node_1.clone()),
                (String::from("y"), unitary_node_1),
            ]),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        assert!(!node.eq_unscheduled(&other))
    }

    #[test]
    fn should_return_false_for_too_much_unitary_nodes() {
        let equation_1 = Equation {
            scope: Scope::Output,
            id: String::from("x"),
            signal_type: Type::Integer,
            expression: StreamExpression::SignalCall {
                signal: Signal {
                    id: String::from("y"),
                    scope: Scope::Local,
                },
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("y"), 0)]),
            },
            location: Location::default(),
        };
        let equation_2 = Equation {
            scope: Scope::Local,
            id: String::from("y"),
            signal_type: Type::Integer,
            expression: StreamExpression::FollowedBy {
                constant: Constant::Integer(0),
                expression: Box::new(StreamExpression::SignalCall {
                    signal: Signal {
                        id: String::from("v"),
                        scope: Scope::Input,
                    },
                    typing: Type::Integer,
                    location: Location::default(),
                    dependencies: Dependencies::from(vec![(String::from("v"), 0)]),
                }),
                typing: Type::Integer,
                location: Location::default(),
                dependencies: Dependencies::from(vec![(String::from("v"), 1)]),
            },
            location: Location::default(),
        };
        let unitary_node = UnitaryNode {
            contracts: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_1.clone(), equation_2.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let node = Node {
            contracts: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1.clone()),
                (String::from("y"), equation_2.clone()),
            ]),
            unitary_nodes: HashMap::from([(String::from("x"), unitary_node)]),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        let unitary_node_1 = UnitaryNode {
            contracts: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("x"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_2.clone(), equation_1.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let unitary_node_2 = UnitaryNode {
            contracts: Default::default(),
            node_id: String::from("test"),
            output_id: String::from("y"),
            inputs: vec![(String::from("v"), Type::Integer)],
            equations: vec![equation_2.clone()],
            memory: Memory::new(),
            location: Location::default(),
            graph: OnceCell::new(),
        };
        let other = Node {
            contracts: Default::default(),
            id: String::from("test"),
            is_component: false,
            inputs: vec![(String::from("v"), Type::Integer)],
            unscheduled_equations: HashMap::from([
                (String::from("x"), equation_1),
                (String::from("y"), equation_2),
            ]),
            unitary_nodes: HashMap::from([
                (String::from("x"), unitary_node_1),
                (String::from("y"), unitary_node_2),
            ]),
            location: Location::default(),
            graph: OnceCell::new(),
        };

        assert!(!node.eq_unscheduled(&other))
    }
}
