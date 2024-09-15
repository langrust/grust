//! HIR [Memory] module.

prelude! {
    hir::stream
}

/// Memory of an unitary node.
///
/// Memory structure for unitary node. It stores initial_valuezed buffers and called unitary nodes'
/// names.
#[derive(Debug, PartialEq, Clone)]
pub struct Memory {
    /// Initialized buffers.
    pub buffers: HashMap<String, Buffer>,
    /// Called unitary nodes' names.
    pub called_nodes: HashMap<usize, CalledNode>,
}

/// Initialized buffer.
///
/// Buffer `initial_value`-ized by a constant.
#[derive(Debug, PartialEq, Clone)]
pub struct Buffer {
    /// Buffered id.
    pub id: usize,
    /// Buffered identifier.
    pub identifier: String,
    /// Buffer type.
    pub typing: Typ,
    /// Buffer initial value.
    pub initial_expression: stream::Expr,
}

/// Called unitary node' name.
///
/// Unitary node's name is composed of the name of the mother node and the name of the called output
/// signal.
#[derive(Debug, PartialEq, Clone)]
pub struct CalledNode {
    /// Node name.
    pub node_id: usize,
}

impl Memory {
    /// Creates empty memory.
    ///
    /// ```rust
    /// # compiler::prelude! { hir::memory::Memory }
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

    /// Adds an initialized buffer to memory.
    pub fn add_buffer(&mut self, id: usize, name: String, typing: Typ, constante: stream::Expr) {
        if let Some(Buffer {
            initial_expression: other_constante,
            ..
        }) = self.buffers.get_mut(&name)
        {
            let default_cst = hir::stream::Kind::expr(hir::expr::Kind::constant(Constant::Default));
            if other_constante.kind.eq(&default_cst) {
                *other_constante = constante
            } else if constante.kind.ne(&default_cst) {
                // todo: make it an error
                assert!(other_constante == &constante, "different init values")
            }
        } else {
            self.buffers.insert(
                name.clone(),
                Buffer {
                    id,
                    identifier: name,
                    typing,
                    initial_expression: constante,
                },
            );
        }
    }

    /// Adds called node to memory.
    pub fn add_called_node(&mut self, memory_id: usize, node_id: usize) {
        let _unique = self.called_nodes.insert(memory_id, CalledNode { node_id });
        debug_assert!(_unique.is_none());
    }
}
impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
