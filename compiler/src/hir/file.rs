use crate::common::location::Location;
use crate::hir::{function::Function, interface::Interface, node::Node, typedef::Typedef};

#[derive(Debug, PartialEq)]
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
    /// Program interface. It represents the system.
    pub interface: Option<Interface>,
    /// Program location.
    pub location: Location,
}

impl File {
    /// Tell if there is no FBY expression.
    pub fn no_fby(&self) -> bool {
        self.nodes.iter().all(|node| node.no_fby())
            && self
                .component
                .as_ref()
                .map_or(true, |component| component.no_fby())
    }
    /// Tell if it is in normal form.
    pub fn is_normal_form(&self) -> bool {
        self.nodes.iter().all(|node| node.is_normal_form())
            && self
                .component
                .as_ref()
                .map_or(true, |component| component.is_normal_form())
    }
    /// Tell if there is no node application.
    pub fn no_node_application(&self) -> bool {
        self.nodes.iter().all(|node| node.no_node_application())
            && self
                .component
                .as_ref()
                .map_or(true, |component| component.no_node_application())
    }
}
