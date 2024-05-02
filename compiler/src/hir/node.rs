use petgraph::graphmap::DiGraphMap;
use std::collections::HashMap;

use crate::common::{label::Label, location::Location};
use crate::hir::{
    contract::Contract, statement::Statement, stream_expression::StreamExpression,
    unitary_node::UnitaryNode,
};

#[derive(Debug, Clone)]
/// LanGRust node HIR.
pub struct Node {
    /// Node identifier.
    pub id: usize,
    /// Node's unscheduled equations.
    pub unscheduled_equations: HashMap<usize, Statement<StreamExpression>>,
    /// Unitary output nodes generated from this node.
    pub unitary_nodes: HashMap<usize, UnitaryNode>,
    /// Node's contract.
    pub contract: Contract,
    /// Node location.
    pub location: Location,
    /// Node dependency graph.
    pub graph: DiGraphMap<usize, Label>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.unscheduled_equations == other.unscheduled_equations
            && self.unitary_nodes == other.unitary_nodes
            && self.contract == other.contract
            && self.location == other.location
            && self.eq_graph(other)
    }
}

impl Node {
    fn eq_graph(&self, other: &Node) -> bool {
        let graph_nodes = self.graph.nodes();
        let other_nodes = other.graph.nodes();
        let graph_edges = self.graph.all_edges();
        let other_edges = other.graph.all_edges();
        graph_nodes.eq(other_nodes) && graph_edges.eq(other_edges)
    }

    /// Tell if there is no FBY expression.
    pub fn no_fby(&self) -> bool {
        self.unitary_nodes
            .iter()
            .all(|(_, unitary_node)| unitary_node.no_fby())
    }
    /// Tell if it is in normal form.
    pub fn is_normal_form(&self) -> bool {
        self.unitary_nodes
            .iter()
            .all(|(_, unitary_node)| unitary_node.is_normal_form())
    }
    /// Tell if there is no node application.
    pub fn no_node_application(&self) -> bool {
        self.unitary_nodes
            .iter()
            .all(|(_, unitary_node)| unitary_node.no_node_application())
    }
}
