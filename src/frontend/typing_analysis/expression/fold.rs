use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::hir::expression::{Expression, ExpressionKind};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the fold expression.
    pub fn typing_fold(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            ExpressionKind::Fold {
                ref mut expression,
                ref mut initialization_expression,
                ref mut function_expression,
            } => {
                // type the expression
                expression.typing(symbol_table, errors)?;

                // verify it is an array
                match expression.get_type().unwrap() {
                    Type::Array(element_type, _) => {
                        // type the initialization expression
                        initialization_expression.typing(symbol_table, errors)?;
                        let initialization_type = initialization_expression.get_type().unwrap();

                        // type the function expression
                        function_expression.typing(symbol_table, errors)?;
                        let function_type = function_expression.get_type_mut().unwrap();

                        // apply the function type to the type of the initialization and array's elements
                        let new_type = function_type.apply(
                            vec![initialization_type.clone(), *element_type.clone()],
                            self.location.clone(),
                            errors,
                        )?;

                        // check the new type is equal to the initialization type
                        new_type.eq_check(initialization_type, self.location.clone(), errors)?;

                        self.typing = Some(new_type);
                        Ok(())
                    }
                    given_type => {
                        let error = Error::ExpectArray {
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
