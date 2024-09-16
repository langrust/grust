prelude! {
    hir::{ IdentifierCreator, Memory, stream },
}

use super::Union;

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
        context_map: &mut HashMap<usize, Union<usize, stream::Expr>>,
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
                let _unique = context_map.insert(*memory_id, Union::I1(fresh_id));
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
        context_map: &HashMap<usize, Union<usize, stream::Expr>>,
        symbol_table: &SymbolTable,
    ) -> Memory {
        let buffers = self
            .buffers
            .iter()
            .map(|(name, buffer)| {
                let mut new_buffer = buffer.clone();
                if let Some(element) = context_map.get(&buffer.id) {
                    match element {
                        Union::I1(new_id)
                        | Union::I2(stream::Expr {
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
                        Union::I2(_) => unreachable!(),
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
                        Union::I1(new_id)
                        | Union::I2(stream::Expr {
                            kind:
                                stream::Kind::Expression {
                                    expression: hir::expr::Kind::Identifier { id: new_id },
                                },
                            ..
                        }) => (new_id.clone(), called_node.clone()),
                        Union::I2(_) => unreachable!(),
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
