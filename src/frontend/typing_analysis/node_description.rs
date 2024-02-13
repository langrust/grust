use std::collections::HashMap;

use crate::common::r#type::Type;

#[derive(Debug, PartialEq)]
/// The description of a node's signals.
pub struct NodeDescription {
    /// Is true when the node is a component.
    pub is_component: bool,
    /// Node's input signals.
    pub inputs: Vec<(String, Type)>,
    /// Node's output signals.
    pub outputs: HashMap<String, Type>,
    /// Node's local signals.
    pub locals: HashMap<String, Type>,
}
