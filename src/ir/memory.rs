use crate::common::{constant::Constant, type_system::Type};

/// Memory of an unitary node.
///
/// Memory structure for unitary node.
/// It stores initialzed buffers and called unitary nodes' names.
pub struct Memory {
    /// Initialized buffers.
    pub buffers: Vec<Buffer>,
    /// Called unitary nodes' names.
    pub called_nodes: Vec<CalledNode>,
}

/// Initialized buffer.
///
/// Buffer initialized by a constant.
pub struct Buffer {
    id: String,
    typing: Type,
    initial_value: Constant,
}

/// Called unitary node' name.
///
/// Unitary node's name is composed of the name of the mother
/// node and the name of the called output signal.
pub struct CalledNode {
    node_id: String,
    signal_id: String,
}
