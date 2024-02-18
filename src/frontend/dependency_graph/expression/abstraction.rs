use crate::{
    error::TerminationError,
    hir::{expression::ExpressionKind, stream_expression::StreamExpression},
};

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of an abstraction stream expression.
    pub fn compute_abstraction_dependencies(
        &self,
    ) -> Result<Vec<(usize, usize)>, TerminationError> {
        match self {
            // no dependencies for abstraction
            ExpressionKind::Abstraction { .. } => Ok(vec![]),
            _ => unreachable!(),
        }
    }
}
