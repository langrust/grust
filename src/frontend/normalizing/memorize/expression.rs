use crate::hir::{
    contract::Contract, expression::ExpressionKind, identifier_creator::IdentifierCreator,
    memory::Memory, stream_expression::StreamExpression,
};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
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
        match self {
            ExpressionKind::Constant { .. }
            | ExpressionKind::Identifier { .. }
            | ExpressionKind::Abstraction { .. }
            | ExpressionKind::Enumeration { .. } => (),
            ExpressionKind::Application {
                function_expression,
                inputs,
            } => {
                function_expression.memorize(
                    signal_id,
                    identifier_creator,
                    memory,
                    contract,
                    symbol_table,
                );
                inputs.iter_mut().for_each(|expression| {
                    expression.memorize(
                        signal_id,
                        identifier_creator,
                        memory,
                        contract,
                        symbol_table,
                    )
                })
            }
            ExpressionKind::Structure { fields, .. } => {
                fields.iter_mut().for_each(|(_, expression)| {
                    expression.memorize(
                        signal_id,
                        identifier_creator,
                        memory,
                        contract,
                        symbol_table,
                    )
                })
            }
            ExpressionKind::Array { elements } => elements.iter_mut().for_each(|expression| {
                expression.memorize(
                    signal_id,
                    identifier_creator,
                    memory,
                    contract,
                    symbol_table,
                )
            }),
            ExpressionKind::Match { expression, arms } => {
                expression.memorize(
                    signal_id,
                    identifier_creator,
                    memory,
                    contract,
                    symbol_table,
                );
                arms.iter_mut().for_each(|(_, option, block, expression)| {
                    debug_assert!(block.is_empty());
                    option.as_mut().map(|expression| {
                        expression.memorize(
                            signal_id,
                            identifier_creator,
                            memory,
                            contract,
                            symbol_table,
                        )
                    });
                    expression.memorize(
                        signal_id,
                        identifier_creator,
                        memory,
                        contract,
                        symbol_table,
                    )
                })
            }
            ExpressionKind::When {
                option,
                present,
                present_body,
                default,
                default_body,
                ..
            } => {
                debug_assert!(present_body.is_empty());
                debug_assert!(default_body.is_empty());
                option.memorize(
                    signal_id,
                    identifier_creator,
                    memory,
                    contract,
                    symbol_table,
                );
                present.memorize(
                    signal_id,
                    identifier_creator,
                    memory,
                    contract,
                    symbol_table,
                );
                default.memorize(
                    signal_id,
                    identifier_creator,
                    memory,
                    contract,
                    symbol_table,
                );
            }
            ExpressionKind::FieldAccess { expression, .. } => expression.memorize(
                signal_id,
                identifier_creator,
                memory,
                contract,
                symbol_table,
            ),
            ExpressionKind::TupleElementAccess { expression, .. } => expression.memorize(
                signal_id,
                identifier_creator,
                memory,
                contract,
                symbol_table,
            ),
            ExpressionKind::Map {
                expression,
                function_expression,
            } => {
                expression.memorize(
                    signal_id,
                    identifier_creator,
                    memory,
                    contract,
                    symbol_table,
                );
                function_expression.memorize(
                    signal_id,
                    identifier_creator,
                    memory,
                    contract,
                    symbol_table,
                )
            }
            ExpressionKind::Fold {
                expression,
                initialization_expression,
                function_expression,
            } => {
                expression.memorize(
                    signal_id,
                    identifier_creator,
                    memory,
                    contract,
                    symbol_table,
                );
                initialization_expression.memorize(
                    signal_id,
                    identifier_creator,
                    memory,
                    contract,
                    symbol_table,
                );
                function_expression.memorize(
                    signal_id,
                    identifier_creator,
                    memory,
                    contract,
                    symbol_table,
                )
            }
            ExpressionKind::Sort {
                expression,
                function_expression,
            } => {
                expression.memorize(
                    signal_id,
                    identifier_creator,
                    memory,
                    contract,
                    symbol_table,
                );
                function_expression.memorize(
                    signal_id,
                    identifier_creator,
                    memory,
                    contract,
                    symbol_table,
                )
            }
            ExpressionKind::Zip { arrays } => arrays.iter_mut().for_each(|expression| {
                expression.memorize(
                    signal_id,
                    identifier_creator,
                    memory,
                    contract,
                    symbol_table,
                )
            }),
        }
    }
}
