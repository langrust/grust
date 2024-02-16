use std::collections::HashMap;

use crate::{
    common::scope::Scope,
    hir::{
        expression::ExpressionKind,
        identifier_creator::IdentifierCreator,
        memory::Memory,
        stream_expression::{StreamExpression, StreamExpressionKind},
    },
    symbol_table::SymbolTable,
};

use super::Union;

impl Memory {
    /// Add the buffer and called_node identifier to the identifier creator.
    ///
    /// It will add the buffer and called_node identifier to the identifier creator.
    /// If the identifier already exists, then the new identifer created by
    /// the identifier creator will be added to the renaming context.
    pub fn add_necessary_renaming(
        &self,
        identifier_creator: &mut IdentifierCreator,
        context_map: &mut HashMap<usize, Union<usize, StreamExpression>>,
        symbol_table: &mut SymbolTable,
    ) {
        self.buffers.keys().for_each(|id| {
            let name = symbol_table.get_name(id);
            let new_name =
                identifier_creator.new_identifier(String::new(), name.clone(), String::new());
            if &new_name != name {
                let new_id = symbol_table
                    .insert_signal(new_name, Scope::Memory, todo!(), true, todo!(), todo!())
                    .expect("do another function");
                assert!(context_map.insert(id.clone(), Union::I1(new_id)).is_none());
            }
        });
        self.called_nodes.keys().for_each(|id| {
            let name = symbol_table.get_name(id);
            let new_name =
                identifier_creator.new_identifier(String::new(), name.clone(), String::new());
            if &new_name != name {
                let new_id = symbol_table
                    .insert_signal(new_name, Scope::Memory, todo!(), true, todo!(), todo!())
                    .expect("do another function");
                assert!(context_map.insert(id.clone(), Union::I1(new_id)).is_none());
            }
        })
    }

    /// Replace identifier occurence by element in context.
    ///
    /// It will return a new memory where the expression has been modified
    /// according to the context:
    /// - if an identifier is mapped to another identifier, then rename all
    /// occurence of the identifier by the new one
    /// - if the identifer is mapped to an expression, then replace all call to
    /// the identifier by the expression
    ///
    /// # Example
    ///
    /// With a context `[x -> a, y -> b/2, z -> c]`, a call to the function
    /// with the equation `z = x + y` will return `c = a + b/2`.
    pub fn replace_by_context(
        &self,
        context_map: &HashMap<usize, Union<usize, StreamExpression>>,
    ) -> Memory {
        let buffers = self
            .buffers
            .iter()
            .map(|(buffer_id, buffer)| {
                let mut new_buffer = buffer.clone();
                new_buffer.expression.replace_by_context(context_map);

                if let Some(element) = context_map.get(buffer_id) {
                    match element {
                        Union::I1(new_id)
                        | Union::I2(StreamExpression {
                            kind:
                                StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Identifier { id: new_id },
                                },
                            ..
                        }) => (new_id.clone(), new_buffer),
                        Union::I2(_) => unreachable!(),
                    }
                } else {
                    (buffer_id.clone(), new_buffer)
                }
            })
            .collect();

        let called_nodes = self
            .called_nodes
            .iter()
            .map(|(called_node_id, called_node)| {
                if let Some(element) = context_map.get(called_node_id) {
                    match element {
                        Union::I1(new_id)
                        | Union::I2(StreamExpression {
                            kind:
                                StreamExpressionKind::Expression {
                                    expression: ExpressionKind::Identifier { id: new_id },
                                },
                            ..
                        }) => (new_id.clone(), called_node.clone()),
                        Union::I2(_) => unreachable!(),
                    }
                } else {
                    (called_node_id.clone(), called_node.clone())
                }
            })
            .collect();

        Memory {
            buffers,
            called_nodes,
        }
    }

    /// Remove called node from memory.
    pub fn remove_called_node(&mut self, called_node_id: &usize) {
        self.called_nodes.remove(called_node_id);
    }

    /// Combine two memories.
    pub fn combine(&mut self, other: Memory) {
        self.buffers.extend(other.buffers);
        self.called_nodes.extend(other.called_nodes);
    }
}
