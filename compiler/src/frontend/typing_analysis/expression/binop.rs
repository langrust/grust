use crate::common::{location::Location, r#type::Type};
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::expression::ExpressionKind;
use crate::symbol_table::SymbolTable;

impl<E> ExpressionKind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Type] to the binop expression.
    pub fn typing_binop(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Type, TerminationError> {
        match self {
            // an binop expression type is the result of the binop
            // of the inputs types to the abstraction/function type
            ExpressionKind::Binop {
                op,
                left_expression,
                right_expression,
            } => {
                // get expressions type
                left_expression.typing(symbol_table, errors)?;
                let left_expression_type = left_expression.get_type().unwrap().clone();
                right_expression.typing(symbol_table, errors)?;
                let right_expression_type = right_expression.get_type().unwrap().clone();

                // get binop type
                let mut binop_type = op.get_type();

                binop_type.apply(
                    vec![left_expression_type, right_expression_type],
                    location.clone(),
                    errors,
                )
            }
            _ => unreachable!(),
        }
    }
}
