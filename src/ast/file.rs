use crate::util::location::Location;

use super::{
    component::Component, function::Function, node::Node, user_defined_type::UserDefinedType,
};

#[derive(Debug, PartialEq)]
/// Enumerates the different kinds of files in LanGRust.
pub enum File {
    /// A LanGRust [File::Module] is composed of todo!()
    Module {
        // todo!()
        /// Module types.
        user_defined_types: Vec<UserDefinedType>,
        /// Module functions.
        functions: Vec<Function>,
        /// Module nodes. They are functional requirements.
        nodes: Vec<Node>,
        /// Module location.
        location: Location,
    },
    /// A LanGRust [File::Program] is composed of todo!()
    Program {
        // todo!()
        /// Program types.
        user_defined_types: Vec<UserDefinedType>,
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
    /// Get types definitions from a LanGRust file.
    pub fn get_types(self) -> Vec<UserDefinedType> {
        match self {
            File::Module {
                user_defined_types,
                functions: _,
                nodes: _,
                location: _,
            } => user_defined_types,
            File::Program {
                user_defined_types,
                functions: _,
                nodes: _,
                component: _,
                location: _,
            } => user_defined_types,
        }
    }
    /// Get functions from a LanGRust file.
    pub fn get_functions(self) -> Vec<Function> {
        match self {
            File::Module {
                user_defined_types: _,
                functions,
                nodes: _,
                location: _,
            } => functions,
            File::Program {
                user_defined_types: _,
                functions,
                nodes: _,
                component: _,
                location: _,
            } => functions,
        }
    }
    /// Get nodes from a LanGRust file.
    pub fn get_nodes(self) -> Vec<Node> {
        match self {
            File::Module {
                user_defined_types: _,
                functions: _,
                nodes,
                location: _,
            } => nodes,
            File::Program {
                user_defined_types: _,
                functions: _,
                nodes,
                component: _,
                location: _,
            } => nodes,
        }
    }
    /// Get types, functions and nodes from a LanGRust file.
    pub fn get_types_functions_nodes(self) -> (Vec<UserDefinedType>, Vec<Function>, Vec<Node>) {
        match self {
            File::Module {
                user_defined_types,
                functions,
                nodes,
                location: _,
            } => (user_defined_types, functions, nodes),
            File::Program {
                user_defined_types,
                functions,
                nodes,
                component: _,
                location: _,
            } => (user_defined_types, functions, nodes),
        }
    }
    /// Get the location of a LanGRust file.
    pub fn get_location(self) -> Location {
        match self {
            File::Module {
                user_defined_types: _,
                functions: _,
                nodes: _,
                location,
            } => location,
            File::Program {
                user_defined_types: _,
                functions: _,
                nodes: _,
                component: _,
                location,
            } => location,
        }
    }
    /// Add a type definition to a LanGRust file functions.
    pub fn push_type(&mut self, user_defined_type: UserDefinedType) {
        match self {
            File::Module {
                ref mut user_defined_types,
                functions: _,
                nodes: _,
                location: _,
            } => user_defined_types.push(user_defined_type),
            File::Program {
                ref mut user_defined_types,
                functions: _,
                nodes: _,
                component: _,
                location: _,
            } => user_defined_types.push(user_defined_type),
        }
    }
    /// Add a function to a LanGRust file functions.
    pub fn push_function(&mut self, function: Function) {
        match self {
            File::Module {
                user_defined_types: _,
                ref mut functions,
                nodes: _,
                location: _,
            } => functions.push(function),
            File::Program {
                user_defined_types: _,
                ref mut functions,
                nodes: _,
                component: _,
                location: _,
            } => functions.push(function),
        }
    }
    /// Add a node to a LanGRust file nodes.
    pub fn push_node(&mut self, node: Node) {
        match self {
            File::Module {
                user_defined_types: _,
                functions: _,
                ref mut nodes,
                location: _,
            } => nodes.push(node),
            File::Program {
                user_defined_types: _,
                functions: _,
                ref mut nodes,
                component: _,
                location: _,
            } => nodes.push(node),
        }
    }
    /// Change the location of a LanGRust file.
    pub fn set_location(&mut self, new_location: Location) {
        match self {
            File::Module {
                user_defined_types: _,
                functions: _,
                nodes: _,
                ref mut location,
            } => *location = new_location,
            File::Program {
                user_defined_types: _,
                functions: _,
                nodes: _,
                component: _,
                ref mut location,
            } => *location = new_location,
        }
    }
}
