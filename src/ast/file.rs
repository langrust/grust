use crate::util::location::Location;

use super::{
    component::Component,
    node::Node,
    function::Function
};

#[derive(Debug, PartialEq)]

/// Enumerates the different kinds of files in LanGRust.
pub enum File {
    /// A LanGRust [File::Module] is composed of todo!()
    Module{
        // todo!()
        /// Module functions.
        functions: Vec<Function>,
        /// Module nodes. They are functional requirements.
        nodes: Vec<Node>,
        /// Module location.
        location: Location,
    },
    /// A LanGRust [File::Program] is composed of todo!()
    Program{
        // todo!()
        /// Program functions.
        functions: Vec<Function>,
        /// Program nodes. They are functional requirements.
        nodes: Vec<Node>,
        /// Program component. It represents the system.
        component: Component,
        /// Program location.
        location: Location,
    },
}
impl File {
    /// Get functions from a LanGRust file.
    pub fn get_functions(self) -> Vec<Function> {
        match self {
            File::Module { functions, nodes: _, location: _ } => functions,
            File::Program { functions, nodes: _, component: _, location: _ } => functions,
        }
    }
    /// Get nodes from a LanGRust file.
    pub fn get_nodes(self) -> Vec<Node> {
        match self {
            File::Module { functions: _, nodes, location: _ } => nodes,
            File::Program { functions: _, nodes, component: _, location: _ } => nodes,
        }
    }
    /// Get functions and nodes from a LanGRust file.
    pub fn get_functions_nodes(self) -> (Vec<Function>, Vec<Node>) {
        match self {
            File::Module { functions, nodes, location: _ } => (functions, nodes),
            File::Program { functions, nodes, component: _, location: _ } => (functions, nodes),
        }
    }
    /// Get the location of a LanGRust file.
    pub fn get_location(self) -> Location {
        match self {
            File::Module { functions: _, nodes: _, location } => location,
            File::Program { functions: _, nodes: _, component: _, location } => location,
        }
    }
    /// Add a function to a LanGRust file functions.
    pub fn push_function(&mut self, function: Function) {
        match self {
            File::Module { ref mut functions, nodes: _, location: _ } => functions.push(function),
            File::Program { ref mut functions, nodes: _, component: _, location: _ } => functions.push(function),
        }
    }
    /// Add a node to a LanGRust file nodes.
    pub fn push_node(&mut self, node: Node) {
        match self {
            File::Module { functions: _, ref mut nodes, location: _ } => nodes.push(node),
            File::Program { functions: _, ref mut nodes, component: _, location: _ } => nodes.push(node),
        }
    }
    /// Change the location of a LanGRust file.
    pub fn set_location(&mut self, new_location: Location) {
        match self {
            File::Module { functions: _, nodes: _, ref mut location } => *location = new_location,
            File::Program { functions: _, nodes: _, component: _, ref mut location } => *location = new_location,
        }
    }
}