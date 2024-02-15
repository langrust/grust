use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::common::{
    graph::{color::Color, Graph},
    location::Location,
    serialize::ordered_map,
};
use crate::hir::{
    contract::Contract, equation::Equation, once_cell::OnceCell, unitary_node::UnitaryNode,
};

#[derive(Debug, Clone, serde::Serialize)]
/// LanGRust node HIR.
pub struct Node {
    /// Node identifier.
    pub id: usize,
    /// Node's unscheduled equations.    
    #[serde(serialize_with = "ordered_map")]
    pub unscheduled_equations: HashMap<usize, Equation>,
    /// Unitary output nodes generated from this node.
    #[serde(serialize_with = "ordered_map")]
    pub unitary_nodes: HashMap<String, UnitaryNode>,
    /// Node's contract.
    pub contract: Contract,
    /// Node location.
    pub location: Location,
    /// Node dependency graph.
    pub graph: OnceCell<DiGraphMap<String, Label>>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.is_component == other.is_component
            && self.inputs == other.inputs
            && self.unscheduled_equations == other.unscheduled_equations
            && self.unitary_nodes == other.unitary_nodes
            && self.contract == other.contract
            && self.location == other.location
            && self.eq_oncecell_graph(other)
    }
}
