use crate::error::{Error, TerminationError};
use crate::hir::expression::Expression;
use crate::symbol_table::{SymbolKind, SymbolTable};

impl Expression {
    /// Add a [Type] to the call expression.
    pub fn typing_call(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // the type of a call expression in the type of the called element in the context
            Expression::Call {
                id,
                typing,
                location,
            } => {
                let symbol = symbol_table
                    .get_symbol(id)
                    .expect("the identifier should exist");
                match symbol.kind() {
                    SymbolKind::Identifier { typing } => {
                        *typing = typing.clone();
                        Ok(())
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}
