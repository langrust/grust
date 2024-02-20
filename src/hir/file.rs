use crate::common::location::Location;
use crate::hir::{function::Function, node::Node, typedef::Typedef};

#[derive(Debug, PartialEq, serde::Serialize)]
/// A LanGRust [File] is composed of functions nodes,
/// types defined by the user and an optional component.
pub struct File {
    /// Program types.
    pub typedefs: Vec<Typedef>,
    /// Program functions.
    pub functions: Vec<Function>,
    /// Program nodes. They are functional requirements.
    pub nodes: Vec<Node>,
    /// Program component. It represents the system.
    pub component: Option<Node>,
    /// Program location.
    pub location: Location,
}

impl File {
    pub fn no_fby(&self) -> bool {
        self.nodes.iter().all(|node| node.no_fby())
            && self
                .component
                .as_ref()
                .map_or(true, |component| component.no_fby())
    }
    pub fn is_normal_form(&self) -> bool {
        self.nodes.iter().all(|node| node.is_normal_form())
            && self
                .component
                .as_ref()
                .map_or(true, |component| component.is_normal_form())
    }
    pub fn no_node_application(&self) -> bool {
        self.nodes.iter().all(|node| node.no_node_application())
            && self
                .component
                .as_ref()
                .map_or(true, |component| component.no_node_application())
    }
}
