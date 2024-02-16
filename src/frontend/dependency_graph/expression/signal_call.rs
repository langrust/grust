use crate::error::TerminationError;
use crate::hir::{expression::ExpressionKind, stream_expression::StreamExpression};

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a signal call.
    pub fn compute_signal_call_dependencies(
        &self,
    ) -> Result<Vec<(usize, usize)>, TerminationError> {
        match self {
            // signal call depends on called signal with depth of 0
            ExpressionKind::Identifier { id, .. } => Ok(vec![(*id, 0)]),
            _ => unreachable!(),
        }
    }
}
