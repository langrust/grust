use crate::common::{location::Location, user_defined_type::UserDefinedType};
use crate::ir::{function::Function, node::Node};

#[derive(Debug, PartialEq)]
/// A LanGRust [File] is composed of functions nodes,
/// types defined by the user and an optional component.
pub struct File {
    /// Program types.
    pub user_defined_types: Vec<UserDefinedType>,
    /// Program functions.
    pub functions: Vec<Function>,
    /// Program nodes. They are functional requirements.
    pub nodes: Vec<Node>,
    /// Program component. It represents the system.
    pub component: Option<Node>,
    /// Program location.
    pub location: Location,
}
