use crate::ast::{function::Function, node::Node, typedef::Typedef};
use crate::common::location::Location;

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
    pub fn push_typedef(&mut self, typedef: Typedef) {
        self.typedefs.push(typedef)
    }
    pub fn push_function(&mut self, function: Function) {
        self.functions.push(function)
    }
    pub fn push_node(&mut self, node: Node) {
        self.nodes.push(node)
    }
    pub fn set_location(&mut self, location: Location) {
        self.location = location;
    }
    pub fn set_component(&mut self, component: Node) {
        self.component = Some(component);
    }
}
