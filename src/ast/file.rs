use std::collections::HashMap;

use crate::ast::{function::Function, global_context, node::Node, typedef::Typedef};
use crate::common::{context::Context, location::Location, r#type::Type};
use crate::error::{Error, TerminationError};

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
