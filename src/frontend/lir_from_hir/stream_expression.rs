use crate::{
    common::{convert_case::camel_case, r#type::Type},
    hir::stream_expression::{StreamExpression, StreamExpressionKind},
    lir::expression::Expression as LIRExpression,
    symbol_table::SymbolTable,
};

use super::LIRFromHIR;

impl LIRFromHIR for StreamExpression {
    type LIR = LIRExpression;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        match self.kind {
            StreamExpressionKind::UnitaryNodeApplication {
                node_id,
                inputs,
                output_id,
            } => {
                let name = symbol_table.get_name(&node_id).clone();
                LIRExpression::NodeCall {
                    node_identifier: name.clone(),
                    input_name: camel_case(&format!("{name}Input")),
                    input_fields: inputs
                        .into_iter()
                        .map(|(id, expression)| {
                            (
                                symbol_table.get_name(&id).clone(),
                                expression.lir_from_hir(symbol_table),
                            )
                        })
                        .collect(),
                }
            }
            StreamExpressionKind::FollowedBy { .. }
            | StreamExpressionKind::NodeApplication { .. } => {
                unreachable!()
            }
            StreamExpressionKind::Expression { expression } => {
                expression.lir_from_hir(symbol_table)
            }
        }
    }

    fn get_type(&self) -> Option<&Type> {
        self.typing.as_ref()
    }

    fn is_if_then_else(&self, symbol_table: &SymbolTable) -> bool {
        match &self.kind {
            StreamExpressionKind::Expression { expression } => {
                expression.is_if_then_else(symbol_table)
            }
            _ => false,
        }
    }
}
