use crate::common::{location::Location, r#type::Type};
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::expression::ExpressionKind;
use crate::symbol_table::SymbolTable;

impl<E> ExpressionKind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Type] to the unop expression.
    pub fn typing_unop(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Type, TerminationError> {
        match self {
            // an unop expression type is the result of the unop
            // of the inputs types to the abstraction/function type
            ExpressionKind::Unop { op, expression } => {
                // get expression type
                expression.typing(symbol_table, errors)?;
                let expression_type = expression.get_type().unwrap().clone();

                // get unop type
                let mut unop_type = op.get_type();

                unop_type.apply(vec![expression_type], location.clone(), errors)
            }
            _ => unreachable!(),
        }
    }
}
