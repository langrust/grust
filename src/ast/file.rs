use crate::util::location::Location;

use super::{component::Component, node::Node};

#[derive(Debug, PartialEq)]

/// Enumerates the different kinds of files in LanGRust.
pub enum File {
    /// A LanGRust [File::Module] is composed of todo!()
    Module{
        // todo!()
        /// Module nodes. They are functional requirements.
        nodes: Vec<Node>,
        /// Module location.
        location: Location,
    },
    /// A LanGRust [File::Program] is composed of todo!()
    Program{
        // todo!()
        /// Program nodes. They are functional requirements.
        nodes: Vec<Node>,
        /// Program component. It represents the system.
        component: Component,
        /// Program location.
        location: Location,
    },
}
impl File {
    /// Get nodes from a LanGRust file.
    pub fn get_nodes(self) -> Vec<Node> {
        match self {
            File::Module { nodes, location: _ } => nodes,
            File::Program { nodes, component: _, location: _ } => nodes,
        }
    }
    /// Get the location of a LanGRust file.
    pub fn get_location(self) -> Location {
        match self {
            File::Module { nodes: _, location } => location,
            File::Program { nodes: _, component: _, location } => location,
        }
    }
    /// Add a node to a LanGRust file nodes.
    pub fn push_node(&mut self, node: Node) {
        match self {
            File::Module { ref mut nodes, location: _ } => nodes.push(node),
            File::Program { ref mut nodes, component: _, location: _ } => nodes.push(node),
        }
    }
    /// Change the location of a LanGRust file.
    pub fn set_location(&mut self, new_location: Location) {
        match self {
            File::Module { nodes: _, ref mut location } => *location = new_location,
            File::Program { nodes: _, component: _, ref mut location } => *location = new_location,
        }
    }
}