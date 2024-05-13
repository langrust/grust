use crate::common::operator::OtherOperator;
use crate::common::{location::Location, r#type::Type};
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::expression::ExpressionKind;
use crate::symbol_table::SymbolTable;

impl<E> ExpressionKind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Type] to the if_then_else expression.
    pub fn typing_if_then_else(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Type, TerminationError> {
        match self {
            // an if_then_else expression type is the result of the if_then_else
            // of the inputs types to the abstraction/function type
            ExpressionKind::IfThenElse {
                expression,
                true_expression,
                false_expression,
            } => {
                // get expressions type
                expression.typing(symbol_table, errors)?;
                let expression_type = expression.get_type().unwrap().clone();
                true_expression.typing(symbol_table, errors)?;
                let true_expression_type = true_expression.get_type().unwrap().clone();
                false_expression.typing(symbol_table, errors)?;
                let false_expression_type = false_expression.get_type().unwrap().clone();

                // get if_then_else type
                let mut if_then_else_type = OtherOperator::IfThenElse.get_type();

                if_then_else_type.apply(
                    vec![expression_type, true_expression_type, false_expression_type],
                    location.clone(),
                    errors,
                )
            }
            _ => unreachable!(),
        }
    }
}
