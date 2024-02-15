use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::hir::expression::{Expression, ExpressionKind};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the when expression.
    pub fn typing_when(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
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
                        // TODO: add type to id
                        present.typing(symbol_table, errors)?;
                        default.typing(symbol_table, errors)?;

                        let present_type = present.get_type().unwrap();
                        let default_type = default.get_type().unwrap();

                        self.typing = Some(present_type.clone());
                        default_type.eq_check(present_type, self.location.clone(), errors)
                    }
                    _ => {
                        let error = Error::ExpectOption {
                            given_type: option_type.clone(),
                            location: self.location.clone(),
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
