use crate::{
    common::label::Label,
    error::TerminationError,
    hir::{expression::ExpressionKind, stream_expression::StreamExpression},
};

impl ExpressionKind<StreamExpression> {
    /// Compute dependencies of an enumeration stream expression.
    pub fn compute_enumeration_dependencies(
        &self,
    ) -> Result<Vec<(usize, Label)>, TerminationError> {
        match self {
            // no dependencies for enumeration stream expression
            ExpressionKind::Enumeration { .. } => Ok(vec![]),
            _ => unreachable!(),
        }
    }
}
