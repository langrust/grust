use std::collections::HashMap;

use once_cell::sync::OnceCell;

use crate::common::{
    graph::{color::Color, Graph},
    location::Location,
    r#type::Type,
};
use crate::hir::{equation::Equation, unitary_node::UnitaryNode};

#[derive(Debug, PartialEq, Clone)]
/// LanGRust node AST.
pub struct Node {
    /// Node identifier.
    pub id: String,
    /// Is true when the node is a component.
    pub is_component: bool,
    /// Node's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Node's unscheduled equations.
    pub unscheduled_equations: HashMap<String, Equation>,
    /// Unitary output nodes generated from this node.
    pub unitary_nodes: HashMap<String, UnitaryNode>,
    /// Node location.
    pub location: Location,
    /// Node dependency graph.
    pub graph: OnceCell<Graph<Color>>,
}
