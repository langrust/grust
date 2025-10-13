use crate::common::label::Label;
use crate::common::scope::Scope;
use crate::hir::contract::Contract;
use crate::hir::expression::ExpressionKind;
use crate::hir::{
    dependencies::Dependencies,
    identifier_creator::IdentifierCreator,
    memory::Memory,
    stream_expression::{StreamExpression, StreamExpressionKind},
};
use crate::symbol_table::SymbolTable;

impl StreamExpression {
    /// Increment memory with expression.
    ///
    /// Store buffer for followed by expressions and unitary node applications.
    /// Transform followed by expressions in signal call.
    ///
    /// # Example
    ///
    /// An expression `0 fby v` increments memory with the buffer
    /// `mem: int = 0 fby v;` and becomes a call to `mem`.
    ///
    /// An expression `my_node(s, x_1).o;` increments memory with the
    /// node call `memmy_node_o_: (my_node, o);` and is unchanged.
    ///
    /// Examples are tested in source.
    pub fn memorize(
        &mut self,
        signal_id: usize,
        identifier_creator: &mut IdentifierCreator,
        memory: &mut Memory,
        contract: &mut Contract,
        symbol_table: &mut SymbolTable,
    ) {
        match &mut self.kind {
            StreamExpressionKind::Expression { expression } => expression.memorize(
                signal_id,
                identifier_creator,
                memory,
                contract,
                symbol_table,
            ),
            StreamExpressionKind::FollowedBy {
                constant,
                expression,
            } => {
                // create fresh identifier for the new memory buffer
                let name = symbol_table.get_name(signal_id);
                let memory_name = identifier_creator.new_identifier(
                    String::from("mem"),
                    name.clone(),
                    String::from(""),
                );
                let typing = self.typing.clone();
                let memory_id =
                    symbol_table.insert_fresh_signal(memory_name, Scope::Memory, typing);

                // add buffer to memory
                memory.add_buffer(memory_id, *constant.clone(), *expression.clone());

                // replace signal id by memory id in contract
                // (Creusot only has access to function input and output in its contract)
                contract.substitution(signal_id, memory_id); // TODO: I do not think this is true

                // replace fby expression by a call to buffer
                self.kind = StreamExpressionKind::Expression {
                    expression: ExpressionKind::Identifier { id: memory_id },
                };
                self.dependencies = Dependencies::from(vec![(memory_id, Label::Weight(0))]);
            }
            StreamExpressionKind::NodeApplication { .. } => unreachable!(),
            StreamExpressionKind::UnitaryNodeApplication {
                node_id,
                inputs,
                output_id,
            } => {
                // create fresh identifier for the new memory buffer
                let node_name = symbol_table.get_name(*node_id);
                let memory_name = identifier_creator.new_identifier(
                    String::from(""),
                    node_name.clone(),
                    String::from(""),
                );
                let memory_id = symbol_table.insert_fresh_signal(memory_name, Scope::Memory, None);

                memory.add_called_node(memory_id, *node_id, *output_id);

                self.dependencies = Dependencies::from(
                    inputs
                        .iter()
                        .flat_map(|(_, expression)| expression.get_dependencies().clone())
                        .collect(),
                );
            }
        }
    }
}
