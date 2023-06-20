use std::collections::HashMap;

use crate::common::{constant::Constant, type_system::Type};

/// Memory of an unitary node.
///
/// Memory structure for unitary node.
/// It stores initialzed buffers and called unitary nodes' names.
#[derive(Debug, PartialEq, Clone)]
pub struct Memory {
    /// Initialized buffers.
    pub buffers: HashMap<String, Buffer>,
    /// Called unitary nodes' names.
    pub called_nodes: HashMap<String, CalledNode>,
}

/// Initialized buffer.
///
/// Buffer initialized by a constant.
#[derive(Debug, PartialEq, Clone)]
pub struct Buffer {
    typing: Type,
    initial: Constant,
}

/// Called unitary node' name.
///
/// Unitary node's name is composed of the name of the mother
/// node and the name of the called output signal.
#[derive(Debug, PartialEq, Clone)]
pub struct CalledNode {
    node_id: String,
    signal_id: String,
}

impl Memory {
    /// Create empty memory.
    pub fn new() -> Self {
        Memory {
            buffers: HashMap::new(),
            called_nodes: HashMap::new(),
        }
    }

    /// Add an initialized buffer to memory.
    pub fn add_buffer(&mut self, memory_id: String, initial: Constant) {
        let typing = initial.get_type();
        assert!(self
            .buffers
            .insert(memory_id, Buffer { typing, initial })
            .is_none())
    }

    /// Add called node to memory.
    pub fn add_called_node(&mut self, memory_id: String, node_id: String, signal_id: String) {
        assert!(self
            .called_nodes
            .insert(memory_id, CalledNode { node_id, signal_id })
            .is_none())
    }
}
