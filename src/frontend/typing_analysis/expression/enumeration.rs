use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::hir::expression::{Expression, ExpressionKind};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the enumeration expression.
    pub fn typing_enumeration(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            // the type of the enumeration is the corresponding enumeration type
            ExpressionKind::Enumeration {
                ref enum_id,
                ref elem_id,
            } => {
                // type each field and check their type
                self.typing = Some(Type::Enumeration {
                    name: symbol_table.get_name(enum_id).clone(),
                    id: *enum_id,
                });
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
