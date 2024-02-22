use crate::common::label::Label;
use crate::error::TerminationError;
use crate::hir::{expression::ExpressionKind, stream_expression::StreamExpression};
use crate::symbol_table::SymbolTable;

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of an identifier.
    pub fn compute_identifier_dependencies(
        &self,
        symbol_table: &SymbolTable,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            // identifier depends on called identifier with label weight of 0
            ExpressionKind::Identifier { id, .. } => {
                if symbol_table.is_function(&id) {
                    Ok(vec![])
                } else {
                    Ok(vec![(*id, Label::Weight(0))])
                }
            }
            _ => unreachable!(),
        }
    }
}
