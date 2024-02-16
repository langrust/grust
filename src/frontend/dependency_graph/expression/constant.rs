use crate::{
    error::TerminationError,
    hir::{expression::ExpressionKind, stream_expression::StreamExpression},
};

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of a constant stream expression.
    pub fn compute_constant_dependencies(&self) -> Result<Vec<(usize, usize)>, TerminationError> {
        match self {
            // no dependencies for constant stream expression
            ExpressionKind::Constant { .. } => Ok(vec![]),
            _ => unreachable!(),
        }
    }
}
