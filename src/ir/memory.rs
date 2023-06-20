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
    memory_id: String,
    typing: Type,
    initial_value: Constant,
}

/// Called unitary node' name.
///
/// Unitary node's name is composed of the name of the mother
/// node and the name of the called output signal.
pub struct CalledNode {
    memory_id: String,
    node_id: String,
    signal_id: String,
}

impl Memory {
    /// Add an initialized buffer to memory.
    pub fn add_buffer(&mut self, memory_id: String, initial_value: Constant) {
        self.buffers.push(Buffer {
            memory_id,
            typing: initial_value.get_type(),
            initial_value,
        })
    }

    /// Add called node to memory.
    pub fn add_called_node(&mut self, memory_id: String, node_id: String, signal_id: String) {
        self.called_nodes.push(CalledNode {
            memory_id,
            node_id,
            signal_id,
        })
    }
}
