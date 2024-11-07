//! HIR [Memory] module.

prelude! {}

/// Memory of an unitary node.
///
/// Memory structure for unitary node. It stores buffers and called unitary nodes'
/// names.
#[derive(Debug, PartialEq, Clone)]
pub struct Memory {
    /// Initialized buffers.
    pub buffers: HashMap<String, Buffer>,
    /// Called unitary nodes' names.
    pub called_nodes: HashMap<usize, CalledNode>,
}

impl Memory {
    pub fn get_identifiers(&self) -> impl Iterator<Item = &usize> {
        self.called_nodes.keys()
    }

    /// Add the buffer and called_node identifier to the identifier creator.
    ///
    /// It will add the buffer and called_node identifier to the identifier creator. If the
    /// identifier already exists, then the new identifier created by the identifier creator will be
    /// added to the renaming context.
    pub fn add_necessary_renaming(
        &self,
        identifier_creator: &mut IdentifierCreator,
        context_map: &mut HashMap<usize, Either<usize, stream::Expr>>,
        symbol_table: &mut SymbolTable,
    ) {
        // buffered signals are renamed with their stmts
        // we just rename the called nodes
        self.called_nodes.keys().for_each(|memory_id| {
            let name = symbol_table.get_name(*memory_id);
            let fresh_name = identifier_creator.new_identifier(name);
            if &fresh_name != name {
                let scope = symbol_table.get_scope(*memory_id).clone(); // supposed to be Scope::Local
                debug_assert_eq!(scope, Scope::Local);
                let typing = None;
                let fresh_id = symbol_table.insert_fresh_signal(fresh_name, scope, typing);
                let _unique = context_map.insert(*memory_id, Either::Left(fresh_id));
                debug_assert!(_unique.is_none());
            }
        })
    }

    /// Replace identifier occurrence by element in context.
    ///
    /// It will return a new memory where the expression has been modified
    /// according to the context:
    /// - if an identifier is mapped to another identifier, then rename all
    /// occurrence of the identifier by the new one
    /// - if the identifier is mapped to an expression, then replace all call to
    /// the identifier by the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2, z -> c]`, a call to the function
    /// with the equation `z = x + y` will return `c = a + b/2`.
    pub fn replace_by_context(
        &self,
        context_map: &HashMap<usize, Either<usize, stream::Expr>>,
        symbol_table: &SymbolTable,
    ) -> Memory {
        let buffers = self
            .buffers
            .iter()
            .map(|(name, buffer)| {
                let mut new_buffer = buffer.clone();
                if let Some(element) = context_map.get(&buffer.id) {
                    match element {
                        Either::Left(new_id)
                        | Either::Right(stream::Expr {
                            kind:
                                stream::Kind::Expression {
                                    expression: hir::expr::Kind::Identifier { id: new_id },
                                },
                            ..
                        }) => {
                            let new_name = symbol_table.get_name(*new_id);
                            new_buffer.id = *new_id;
                            new_buffer.identifier = new_name.clone();
                            (new_name.clone(), new_buffer)
                        }
                        Either::Right(_) => unreachable!(),
                    }
                } else {
                    (name.clone(), new_buffer)
                }
            })
            .collect();

        let called_nodes = self
            .called_nodes
            .iter()
            .map(|(memory_id, called_node)| {
                if let Some(element) = context_map.get(memory_id) {
                    match element {
                        Either::Left(new_id)
                        | Either::Right(stream::Expr {
                            kind:
                                stream::Kind::Expression {
                                    expression: hir::expr::Kind::Identifier { id: new_id },
                                },
                            ..
                        }) => (new_id.clone(), called_node.clone()),
                        Either::Right(_) => unreachable!(),
                    }
                } else {
                    (memory_id.clone(), called_node.clone())
                }
            })
            .collect();

        Memory {
            buffers,
            called_nodes,
        }
    }

    /// Remove called node from memory.
    pub fn remove_called_node(&mut self, memory_id: usize) {
        self.called_nodes.remove(&memory_id);
    }

    /// Combine two memories.
    pub fn combine(&mut self, other: Memory) {
        self.buffers.extend(other.buffers);
        self.called_nodes.extend(other.called_nodes);
    }
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
    pub initial_expression: hir::stream::Expr,
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
    /// # compiler_hir::prelude! { }
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
    pub fn add_buffer(
        &mut self,
        id: usize,
        name: String,
        typing: Typ,
        constant: hir::stream::Expr,
    ) {
        if let Some(Buffer {
            initial_expression: other_constant,
            ..
        }) = self.buffers.get_mut(&name)
        {
            let default_cst = hir::stream::Kind::expr(hir::expr::Kind::constant(Constant::Default));
            if other_constant.kind == default_cst {
                *other_constant = constant;
            } else if constant.kind.ne(&default_cst) {
                // todo: make it an error
                assert!(other_constant == &constant, "different init values")
            }
        } else {
            self.buffers.insert(
                name.clone(),
                Buffer {
                    id,
                    identifier: name,
                    typing,
                    initial_expression: constant,
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
