use crate::{
    common::convert_case::camel_case,
    frontend::lir_from_hir::expression::lir_from_hir as expression_lir_from_hir,
    hir::stream_expression::{StreamExpression, StreamExpressionKind},
    lir::expression::Expression as LIRExpression,
    symbol_table::SymbolTable,
};

/// Transform HIR stream expression into LIR expression.
pub fn lir_from_hir(
    stream_expression: StreamExpression,
    symbol_table: &SymbolTable,
) -> LIRExpression {
    match stream_expression.kind {
        StreamExpressionKind::UnitaryNodeApplication {
            node_id,
            inputs,
            output_id,
        } => {
            let name = symbol_table.get_name(&node_id).clone();
            LIRExpression::NodeCall {
                node_identifier: name,
                input_name: camel_case(&format!("{name}Input")),
                input_fields: inputs
                    .into_iter()
                    .map(|(id, expression)| {
                        (
                            symbol_table.get_name(&id).clone(),
                            lir_from_hir(expression, symbol_table),
                        )
                    })
                    .collect(),
            }
        }
        StreamExpressionKind::FollowedBy { .. } | StreamExpressionKind::NodeApplication { .. } => {
            unreachable!()
        }
        StreamExpressionKind::Expression { expression } => {
            expression_lir_from_hir(expression, symbol_table)
        }
    }
}
