prelude! {
    graph::Label,
    hir::{ Contract, Dependencies, IdentifierCreator, Memory, stream },
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
            stream::Kind::FollowedBy {
                constant,
                expression,
            } => {
                // create fresh identifier for the new memory buffer
                let memory_name = identifier_creator.new_identifier("mem");
                let typing = self.typing.clone();
                let memory_id =
                    symbol_table.insert_fresh_signal(memory_name, Scope::Memory, typing);

                // add buffer to memory
                memory.add_buffer(memory_id, *constant.clone(), *expression.clone());

                // replace signal id by memory id in contract
                // (Creusot only has access to function input and output in its contract)
                // contract.substitution(signal_id, memory_id); // TODO: followed by as root expression

                // replace fby expression by a call to buffer
                self.kind = stream::Kind::Expression {
                    expression: hir::expr::Kind::Identifier { id: memory_id },
                };
                self.dependencies = Dependencies::from(vec![(memory_id, Label::Weight(0))]);
            }
            stream::Kind::NodeApplication {
                called_node_id,
                inputs,
                memory_id: node_memory_id,
                ..
            } => {
                debug_assert!(node_memory_id.is_none());

                // create fresh identifier for the new memory buffer
                let node_name = symbol_table.get_name(*called_node_id);
                let memory_name = identifier_creator.new_identifier(&node_name);
                let memory_id = symbol_table.insert_fresh_signal(memory_name, Scope::Memory, None);

                let inputs_map = inputs
                    .iter()
                    .map(|(input_id, expression)| {
                        let mut dependencies = expression.get_dependencies().clone();
                        debug_assert!(dependencies.len() == 1); // normalization makes them identifier expressions
                        let (given_id, _) = dependencies.pop().unwrap();
                        (*input_id, given_id)
                    })
                    .collect::<Vec<_>>();
                memory.add_called_node(memory_id, *called_node_id, inputs_map);

                // put the 'memory_id' of the called node
                *node_memory_id = Some(memory_id);
                self.dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|(_, expression)| expression.get_dependencies().clone())
                        .collect(),
                );
            }
            stream::Kind::SomeEvent { expression } => {
                expression.memorize(identifier_creator, memory, contract, symbol_table);
                self.dependencies = Dependencies::from(expression.get_dependencies().clone());
            }
            stream::Kind::NoneEvent => (),
            stream::Kind::RisingEdge { .. } => unreachable!(),
        }
    }
}
