use crate::ast::{function::Function, interface::Interface, node::Node, typedef::Typedef};
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
    /// Program interface. It represents the system.
    pub interface: Option<Interface>,
    /// Program location.
    pub location: Location,
}

impl File {
    /// Add a [Typedef] to the GRust file.
    pub fn push_typedef(&mut self, typedef: Typedef) {
        self.typedefs.push(typedef)
    }
    /// Add a [Function] to the GRust file.
    pub fn push_function(&mut self, function: Function) {
        self.functions.push(function)
    }
    /// Add a [Node] to the GRust file.
    pub fn push_node(&mut self, node: Node) {
        self.nodes.push(node)
    }
    /// Set GRust file location.
    pub fn set_location(&mut self, location: Location) {
        self.location = location;
    }
    /// Set GRust file component.
    pub fn set_component(&mut self, component: Node) {
        self.component = Some(component);
    }
    /// Set GRust file interface.
    pub fn set_interface(&mut self, interface: Interface) {
        self.interface = Some(interface);
    }
}
