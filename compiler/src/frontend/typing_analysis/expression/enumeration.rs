use crate::common::r#type::Type;
use crate::error::TerminationError;
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::expression::ExpressionKind;
use crate::symbol_table::SymbolTable;

impl<E> ExpressionKind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Type] to the enumeration expression.
    pub fn typing_enumeration(
        &mut self,
        symbol_table: &mut SymbolTable,
    ) -> Result<Type, TerminationError> {
        match self {
            // the type of the enumeration is the corresponding enumeration type
            ExpressionKind::Enumeration { ref enum_id, .. } => {
                // type each field and check their type
                Ok(Type::Enumeration {
                    name: symbol_table.get_name(*enum_id).clone(),
                    id: *enum_id,
                })
            }
            _ => unreachable!(),
        }
    }
}
