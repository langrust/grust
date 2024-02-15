use crate::{
    error::TerminationError,
    hir::{signal::Signal, expression::{Expression, ExpressionKind}},
};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Compute dependencies of a signal call.
    pub fn compute_signal_call_dependencies(&self) -> Result<(), TerminationError> {
        match self.kind {
            // signal call depends on called signal with depth of 0
            ExpressionKind::Identifier {
                id,
                ..
            } => {
                self.dependencies.set(vec![(id.clone(), 0)]);

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

