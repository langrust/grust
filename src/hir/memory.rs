use std::collections::HashMap;

use crate::common::{constant::Constant, r#type::Type, serialize::ordered_map};

use crate::hir::stream_expression::StreamExpression;

/// Memory of an unitary node.
///
/// Memory structure for unitary node.
/// It stores initial_valuezed buffers and called unitary nodes' names.
#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct Memory {
    /// Initialized buffers.
    #[serde(serialize_with = "ordered_map")]
    pub buffers: HashMap<usize, Buffer>,
    /// Called unitary nodes' names.
    #[serde(serialize_with = "ordered_map")]
    pub called_nodes: HashMap<usize, CalledNode>,
}

/// Initialized buffer.
///
/// Buffer initial_valueized by a constant.
#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct Buffer {
    /// Buffer type.
    pub typing: Type,
    /// Buffer initial value.
    pub initial_value: Constant,
    /// Buffer update expression.
    pub expression: StreamExpression,
}

/// Called unitary node' name.
///
/// Unitary node's name is composed of the name of the mother
/// node and the name of the called output signal.
#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct CalledNode {
    /// Node name.
    pub node_id: usize,
    /// Output signal name.
    pub signal_id: usize,
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

    /// Add an initial_valueized buffer to memory.
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use grustine::common::{
    ///     constant::Constant, location::Location, scope::Scope, r#type::Type
    /// };
    /// use grustine::hir::{
    ///     dependencies::Dependencies, memory::Memory, signal::Signal,
    ///     stream_expression::StreamExpression,
    /// };
    ///
    /// let mut memory = Memory::new();
    ///
    /// memory.add_buffer(
    ///     String::from("toto"),
    ///     Constant::Integer(0),
    ///     StreamExpression::SignalCall {
    ///         signal: Signal {
    ///             id: String::from("x"),
    ///             scope: Scope::Local,
    ///         },
    ///         typing: Type::Integer,
    ///         location: Location::default(),
    ///         dependencies: Dependencies::new(),
    ///     });
    ///
    /// assert!(!memory.buffers.is_empty());
    /// assert!(memory.buffers.contains_key(&String::from("toto")));
    /// assert!(memory.called_nodes.is_empty());
    /// ```
    pub fn add_buffer(
        &mut self,
        memory_id: usize,
        initial_value: Constant,
        expression: StreamExpression,
    ) {
        let typing = initial_value.get_type();
        debug_assert!(self
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
    /// use grustine::common::{constant::Constant, r#type::Type, location::Location};
    /// use grustine::hir::{stream_expression::StreamExpression, memory::Memory};
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
    pub fn add_called_node(&mut self, memory_id: usize, node_id: usize, signal_id: usize) {
        debug_assert!(self
            .called_nodes
            .insert(memory_id, CalledNode { node_id, signal_id })
            .is_none())
    }
}
impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
