//! [Memory] module.

prelude! {}

/// Memory of an node.
///
/// It stores buffers and called nodes' names.
#[derive(Debug, PartialEq, Clone)]
pub struct Memory {
    /// Initialized buffers.
    pub buffers: HashMap<Ident, Buffer>,
    /// Called nodes' names.
    pub called_nodes: HashMap<usize, CalledNode>,
    /// Ghost nodes' names.
    pub ghost_nodes: HashMap<usize, GhostNode>,
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
        ctx: &mut Ctx,
    ) {
        // buffered signals are renamed with their stmts
        // we just rename the called nodes
        self.called_nodes.keys().for_each(|memory_id| {
            let name = ctx.get_name(*memory_id);
            let fresh_name = identifier_creator.new_identifier(name.span(), name.to_string());
            if &fresh_name != name {
                // supposed to be Scope::Local
                let scope = ctx.get_scope(*memory_id).clone();
                debug_assert_eq!(scope, Scope::Local);
                let typing = None;
                let fresh_id = ctx.insert_fresh_signal(fresh_name, scope, typing);
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
    ///     occurrence of the identifier by the new one
    /// - if the identifier is mapped to an expression, then replace all call to
    ///     the identifier by the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2, z -> c]`, a call to the function
    /// with the equation `z = x + y` will return `c = a + b/2`.
    pub fn replace_by_context(
        &self,
        context_map: &HashMap<usize, Either<usize, stream::Expr>>,
        ctx: &Ctx,
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
                                    expr: ir1::expr::Kind::Identifier { id: new_id },
                                },
                            ..
                        }) => {
                            let new_name = ctx.get_name(*new_id);
                            new_buffer.id = *new_id;
                            new_buffer.ident = new_name.clone();
                            (new_name.clone(), new_buffer)
                        }
                        Either::Right(_) => noErrorDesc!(),
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
                                    expr: ir1::expr::Kind::Identifier { id: new_id },
                                },
                            ..
                        }) => (*new_id, called_node.clone()),
                        Either::Right(_) => noErrorDesc!(),
                    }
                } else {
                    (*memory_id, called_node.clone())
                }
            })
            .collect();

        let ghost_nodes = self
            .ghost_nodes
            .iter()
            .map(|(memory_id, ghost_node)| {
                if let Some(element) = context_map.get(memory_id) {
                    match element {
                        Either::Left(new_id)
                        | Either::Right(stream::Expr {
                            kind:
                                stream::Kind::Expression {
                                    expr: ir1::expr::Kind::Identifier { id: new_id },
                                },
                            ..
                        }) => (*new_id, ghost_node.clone()),
                        Either::Right(_) => noErrorDesc!(),
                    }
                } else {
                    (*memory_id, ghost_node.clone())
                }
            })
            .collect();

        Memory {
            buffers,
            called_nodes,
            ghost_nodes,
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
#[derive(Debug, PartialEq, Clone)]
pub struct Buffer {
    /// Buffered id.
    pub id: usize,
    /// Buffered identifier.
    pub ident: Ident,
    /// Buffer type.
    pub typing: Typ,
    /// Buffer initial value.
    pub init: ir1::stream::Expr,
}

/// Called node' name.
#[derive(Debug, PartialEq, Clone)]
pub struct CalledNode {
    /// Node name.
    pub node_id: usize,
}

/// Called ghost node' name.
#[derive(Debug, PartialEq, Clone)]
pub struct GhostNode {
    /// Node name.
    pub node_id: usize,
}

impl Memory {
    /// Creates empty memory.
    ///
    /// ```rust
    /// # compiler_ir1::prelude! { }
    /// let memory = Memory::new();
    /// assert!(memory.buffers.is_empty());
    /// assert!(memory.called_nodes.is_empty());
    /// ```
    pub fn new() -> Self {
        Memory {
            buffers: HashMap::new(),
            called_nodes: HashMap::new(),
            ghost_nodes: HashMap::new(),
        }
    }

    /// Adds an initialized buffer to memory.
    pub fn add_buffer(
        &mut self,
        id: usize,
        ident: Ident,
        typing: Typ,
        constant: ir1::stream::Expr,
    ) -> URes {
        if let Some(Buffer {
            init: other_constant,
            ..
        }) = self.buffers.get_mut(&ident)
        {
            if other_constant.is_default_constant() {
                // overwrite default
                *other_constant = constant;
            } else if constant.is_default_constant() {
                // do nothing, an actual value is already there
            } else if other_constant != &constant {
                bail!(@ident.loc() =>
                    "[internal] incompatible initial values for `{}`", ident
                    => | @constant.loc() => "involving this constant",
                    => | @other_constant.loc() => "and this constant",
                );
            }
            Ok(())
        } else {
            self.buffers.insert(
                ident.clone(),
                Buffer {
                    id,
                    ident,
                    typing,
                    init: constant,
                },
            );
            Ok(())
        }
    }

    /// Adds called node to memory.
    pub fn add_called_node(&mut self, memory_id: usize, node_id: usize) {
        let _unique = self.called_nodes.insert(memory_id, CalledNode { node_id });
        debug_assert!(_unique.is_none());
    }

    /// Adds a ghost node to memory.
    pub fn add_ghost_node(&mut self, memory_id: usize, node_id: usize) {
        let _unique = self.ghost_nodes.insert(memory_id, GhostNode { node_id });
        debug_assert!(_unique.is_none());
    }
}
impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
