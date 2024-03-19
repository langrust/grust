use crate::common::{location::Location, r#type::Type};
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::expression::ExpressionKind;
use crate::symbol_table::SymbolTable;

impl<E> ExpressionKind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Type] to the structure expression.
    pub fn typing_structure(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Type, TerminationError> {
        match self {
            // the type of the structure is the corresponding structure type
            // if fields match their expected types
            ExpressionKind::Structure {
                ref id,
                ref mut fields,
            } => {
                // type each field and check their type
                fields
                    .iter_mut()
                    .map(|(id, expression)| {
                        expression.typing(symbol_table, errors)?;
                        let expression_type = expression.get_type().unwrap();
                        let expected_type = symbol_table.get_type(*id);
                        expression_type.eq_check(expected_type, location.clone(), errors)
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                Ok(Type::Structure {
                    name: symbol_table.get_name(*id).clone(),
                    id: *id,
                })
            }
            _ => unreachable!(),
        }
    }
}
