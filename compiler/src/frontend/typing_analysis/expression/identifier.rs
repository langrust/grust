use crate::common::r#type::Type;
use crate::error::TerminationError;
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::expression::ExpressionKind;
use crate::symbol_table::SymbolTable;

impl<E> ExpressionKind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Type] to the call expression.
    pub fn typing_identifier(
        &mut self,
        symbol_table: &mut SymbolTable,
    ) -> Result<Type, TerminationError> {
        match self {
            // the type of a call expression in the type of the called element in the context
            ExpressionKind::Identifier { ref id } => {
                let typing = symbol_table.get_type(*id);
                Ok(typing.clone())
            }
            _ => unreachable!(),
        }
    }
}
