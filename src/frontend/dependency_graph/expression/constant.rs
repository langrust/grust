use crate::{error::TerminationError, hir::expression::{Expression, ExpressionKind}};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Compute dependencies of a constant stream expression.
    pub fn compute_constant_dependencies(&self) -> Result<(), TerminationError> {
        match self.kind {
            // no dependencies for constant stream expression
            ExpressionKind::Constant { .. } => {
                self.dependencies.set(vec![]);
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
