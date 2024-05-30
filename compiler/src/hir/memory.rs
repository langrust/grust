use std::collections::HashMap;

use crate::common::r#type::Type;

use crate::hir::stream_expression::StreamExpression;

/// Memory of an unitary node.
///
/// Memory structure for unitary node.
/// It stores initial_valuezed buffers and called unitary nodes' names.
#[derive(Debug, PartialEq, Clone)]
pub struct Memory {
    /// Initialized buffers.
    pub buffers: HashMap<usize, Buffer>,
    /// Called unitary nodes' names.
    pub called_nodes: HashMap<usize, CalledNode>,
}

/// Initialized buffer.
///
/// Buffer initial_valueized by a constant.
#[derive(Debug, PartialEq, Clone)]
pub struct Buffer {
    /// Buffer type.
    pub typing: Type,
    /// Buffer initial value.
    pub initial_expression: StreamExpression,
    /// Buffer update expression.
    pub expression: StreamExpression,
}

/// Called unitary node' name.
///
/// Unitary node's name is composed of the name of the mother
/// node and the name of the called output signal.
#[derive(Debug, PartialEq, Clone)]
pub struct CalledNode {
    /// Node name.
    pub node_id: usize,
    /// Maps called node inputs with given input identifier.
    pub inputs_map: Vec<(usize, usize)>,
}

impl Memory {
    /// Create empty memory.
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::hir::memory::Memory;
    ///
    /// let memory = Memory::new();
    /// assert!(memory.buffers.is_empty());
    /// assert!(memory.called_nodes.is_empty());
    /// ```
    pub fn new() -> Self {
        Memory {
            buffers: HashMap::new(),
            called_nodes: HashMap::new(),
        }
    }

    /// Add an initialized buffer to memory.
    pub fn add_buffer(
        &mut self,
        memory_id: usize,
        initial_expression: StreamExpression,
        expression: StreamExpression,
    ) {
        let typing = initial_expression.get_type().unwrap().clone();
        debug_assert!(self
            .buffers
            .insert(
                memory_id,
                Buffer {
                    typing,
                    initial_expression,
                    expression
                }
            )
            .is_none())
    }

    /// Add called node to memory.
    pub fn add_called_node(
        &mut self,
        memory_id: usize,
        node_id: usize,
        inputs_map: Vec<(usize, usize)>,
    ) {
        debug_assert!(self
            .called_nodes
            .insert(
                memory_id,
                CalledNode {
                    node_id,
                    inputs_map
                }
            )
            .is_none())
    }
}
impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
