use crate::error::{Error, TerminationError};
use crate::hir::expression::{Expression, ExpressionKind};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the call expression.
    pub fn typing_call(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            // the type of a call expression in the type of the called element in the context
            ExpressionKind::Identifier { ref id } => {
                let typing = symbol_table.get_type(id);
                self.typing = Some(typing.clone());
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
