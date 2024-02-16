use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::expression::ExpressionKind;
use crate::symbol_table::SymbolTable;

impl<E> ExpressionKind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Type] to the abstraction expression.
    pub fn typing_abstraction(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Type, TerminationError> {
        match self {
            // the type of a typed abstraction is computed by adding inputs to
            // the context and typing the function body expression
            ExpressionKind::Abstraction {
                ref inputs,
                ref mut expression,
            } => {
                // type the abstracted expression with the local context
                expression.typing(symbol_table, errors)?;

                // compute abstraction type
                let input_types = inputs
                    .iter()
                    .map(|id| symbol_table.get_type(id).clone())
                    .collect::<Vec<_>>();
                let abstraction_type = Type::Abstract(
                    input_types,
                    Box::new(expression.get_type().unwrap().clone()),
                );

                Ok(abstraction_type)
            }
            _ => unreachable!(),
        }
    }
}
