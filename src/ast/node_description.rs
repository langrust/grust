use std::collections::HashMap;

use crate::ast::type_system::Type;

#[derive(Debug, PartialEq)]
/// The description of a node's signals.
pub struct NodeDescription {
    /// Node's input signals.
    pub inputs: Vec<(String, Type)>,
    /// Node's output signals.
    pub outputs: HashMap<String, Type>,
    /// Node's local signals.
    pub locals: HashMap<String, Type>,
}
