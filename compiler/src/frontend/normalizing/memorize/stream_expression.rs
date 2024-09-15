prelude! {
    hir::{ Contract, IdentifierCreator, Memory, stream },
}

impl stream::Expr {
    /// Increment memory with expression.
    ///
    /// Store buffer for followed by expressions and unitary node applications. Transform followed
    /// by expressions in signal call.
    ///
    /// # Example
    ///
    /// An expression `0 fby v` increments memory with the buffer `mem: int = 0 fby v;` and becomes
    /// a call to `mem`.
    ///
    /// An expression `my_node(s, x_1).o;` increments memory with the node call `memmy_node_o_:
    /// (my_node, o);` and is unchanged.
    ///
    /// Examples are tested in source.
    pub fn memorize(
        &mut self,
        identifier_creator: &mut IdentifierCreator,
        memory: &mut Memory,
        contract: &mut Contract,
        symbol_table: &mut SymbolTable,
    ) {
        match &mut self.kind {
            stream::Kind::Expression { expression } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table)
            }
            stream::Kind::FollowedBy { id, constant } => {
                // add buffer to memory
                let name = symbol_table.get_name(*id);
                let typing = symbol_table.get_type(*id);
                memory.add_buffer(*id, name.clone(), typing.clone(), *constant.clone());
            }
            stream::Kind::NodeApplication {
                called_node_id,
                memory_id: node_memory_id,
                ..
            } => {
                debug_assert!(node_memory_id.is_none());
                // create fresh identifier for the new memory buffer
                let node_name = symbol_table.get_name(*called_node_id);
                let memory_name = identifier_creator.new_identifier(&node_name);
                let memory_id = symbol_table.insert_fresh_signal(memory_name, Scope::Local, None);
                memory.add_called_node(memory_id, *called_node_id);
                // put the 'memory_id' of the called node
                *node_memory_id = Some(memory_id);
            }
            stream::Kind::SomeEvent { expression } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table);
            }
            stream::Kind::NoneEvent => (),
            stream::Kind::RisingEdge { .. } => unreachable!(),
        }
    }
}
