use std::collections::HashMap;

use crate::common::{
    graph::{color::Color, Graph},
    location::Location,
    serialize::ordered_map,
};
use crate::hir::{
    contract::Contract, equation::Equation, once_cell::OnceCell, unitary_node::UnitaryNode,
};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
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
    pub graph: OnceCell<Graph<Color>>,
}
