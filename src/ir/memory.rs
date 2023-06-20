use std::collections::HashMap;

use crate::common::{constant::Constant, type_system::Type};

use crate::ir::stream_expression::StreamExpression;

/// Memory of an unitary node.
///
/// Memory structure for unitary node.
/// It stores initial_valuezed buffers and called unitary nodes' names.
#[derive(Debug, PartialEq, Clone)]
pub struct Memory {
    /// Initialized buffers.
    pub buffers: HashMap<String, Buffer>,
    /// Called unitary nodes' names.
    pub called_nodes: HashMap<String, CalledNode>,
}

/// Initialized buffer.
///
/// Buffer initial_valueized by a constant.
#[derive(Debug, PartialEq, Clone)]
pub struct Buffer {
    typing: Type,
    initial_value: Constant,
    expression: StreamExpression,
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
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::ir::memory::Memory;
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

    /// Add an initial_valueized buffer to memory.
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::common::{constant::Constant, type_system::Type, location::Location};
    /// use grustine::ir::{stream_expression::StreamExpression, memory::Memory};
    ///
    /// let mut memory = Memory::new();
    ///
    /// memory.add_buffer(
    ///     String::from("toto"),
    ///     Constant::Integer(0),
    ///     StreamExpression::SignalCall {
    ///         id: String::from("x"),
    ///         typing: Type::Integer,
    ///         location: Location::default(),
    ///     });
    ///
    /// assert!(!memory.buffers.is_empty());
    /// assert!(memory.buffers.contains_key(&String::from("toto")));
    /// assert!(memory.called_nodes.is_empty());
    /// ```
    pub fn add_buffer(
        &mut self,
        memory_id: String,
        initial_value: Constant,
        expression: StreamExpression,
    ) {
        let typing = initial_value.get_type();
        assert!(self
            .buffers
            .insert(
                memory_id,
                Buffer {
                    typing,
                    initial_value,
                    expression
                }
            )
            .is_none())
    }

    /// Add called node to memory.
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::common::{constant::Constant, type_system::Type, location::Location};
    /// use grustine::ir::{stream_expression::StreamExpression, memory::Memory};
    ///
    /// let mut memory = Memory::new();
    ///
    /// memory.add_called_node(
    ///     String::from("toto"),
    ///     String::from("toto_node"),
    ///     String::from("toto_signal")
    /// );
    ///
    /// assert!(!memory.called_nodes.is_empty());
    /// assert!(memory.called_nodes.contains_key(&String::from("toto")));
    /// assert!(memory.buffers.is_empty());
    /// ```
    pub fn add_called_node(&mut self, memory_id: String, node_id: String, signal_id: String) {
        assert!(self
            .called_nodes
            .insert(memory_id, CalledNode { node_id, signal_id })
            .is_none())
    }
}
