use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::hir::expression::{Expression, ExpressionKind};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the tuple element access expression.
    pub fn typing_tuple_element_access(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            ExpressionKind::TupleElementAccess {
                ref mut expression,
                ref element_number,
            } => {
                expression.typing(symbol_table, errors)?;

                match expression.get_type().unwrap() {
                    Type::Tuple(elements_type) => {
                        let option_element_type = elements_type.get(*element_number);
                        if let Some(element_type) = option_element_type {
                            self.typing = Some(element_type.clone());
                            Ok(())
                        } else {
                            let error = Error::IndexOutOfBounds {
                                location: self.location.clone(),
                            };
                            errors.push(error);
                            Err(TerminationError)
                        }
                    }
                    given_type => {
                        let error = Error::ExpectTuple {
                            given_type: given_type.clone(),
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
