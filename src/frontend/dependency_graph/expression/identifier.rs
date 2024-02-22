use crate::common::label::Label;
use crate::error::TerminationError;
use crate::hir::{expression::ExpressionKind, stream_expression::StreamExpression};

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of an identifier.
    pub fn compute_identifier_dependencies(&self) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            // identifier depends on called identifier with label weight of 0
            ExpressionKind::Identifier { id, .. } => Ok(vec![(*id, Label::Weight(0))]),
            _ => unreachable!(),
        }
    }
}
