use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::expression::ExpressionKind;
use crate::symbol_table::SymbolTable;

impl<E> ExpressionKind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Type] to the tuple expression.
    pub fn typing_tuple(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Type, TerminationError> {
        match self {
            // an tuple is composed of `n` elements of the different type `t_k` and
            // its type is `(t_1, ..., t_n)`
            ExpressionKind::Tuple { ref mut elements } => {
                debug_assert!(elements.len() >= 1);

                let elements_types = elements
                    .iter_mut()
                    .map(|element| {
                        element.typing(symbol_table, errors)?;
                        Ok(element.get_type().expect("should be typed").clone())
                    })
                    .collect::<Vec<Result<Type, TerminationError>>>()
                    .into_iter()
                    .collect::<Result<Vec<Type>, TerminationError>>()?;

                let tuple_type = Type::Tuple(elements_types);

                Ok(tuple_type)
            }
            _ => unreachable!(),
        }
    }
}
