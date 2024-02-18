use crate::common::{location::Location, r#type::Type};
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::expression::ExpressionKind;
use crate::symbol_table::SymbolTable;

impl<E> ExpressionKind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Type] to the when expression.
    pub fn typing_when(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Type, TerminationError> {
        match self {
            // the type of a when expression is the type of both the default and
            // the present expressions
            ExpressionKind::When {
                ref id,
                ref mut option,
                ref mut present,
                ref mut default,
                ..
            } => {
                option.typing(symbol_table, errors)?;

                let option_type = option.get_type().unwrap();
                match option_type {
                    Type::Option(unwraped_type) => {
                        symbol_table.set_type(id, *unwraped_type.clone());
                        present.typing(symbol_table, errors)?;
                        default.typing(symbol_table, errors)?;

                        let present_type = present.get_type().unwrap();
                        let default_type = default.get_type().unwrap();

                        default_type.eq_check(present_type, location.clone(), errors)?;
                        Ok(present_type.clone())
                    }
                    _ => {
                        let error = Error::ExpectOption {
                            given_type: option_type.clone(),
                            location: location.clone(),
                        };
                        errors.push(error);
                        Err(TerminationError)
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
