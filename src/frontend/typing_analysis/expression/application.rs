use crate::error::{Error, TerminationError};
use crate::hir::expression::{Expression, ExpressionKind};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the application expression.
    pub fn typing_application(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            // an application expression type is the result of the application
            // of the inputs types to the abstraction/function type
            ExpressionKind::Application {
                ref mut function_expression,
                ref mut inputs,
            } => {
                // type all inputs
                inputs
                    .iter_mut()
                    .map(|input| input.typing(symbol_table, errors))
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                let input_types = inputs
                    .iter()
                    .map(|input| input.get_type().unwrap().clone())
                    .collect::<Vec<_>>();

                // type the function expression
                function_expression.typing(symbol_table, errors)?;

                // compute the application type
                let application_type = function_expression.get_type_mut().unwrap().apply(
                    input_types,
                    self.location.clone(),
                    errors,
                )?;

                self.typing = Some(application_type);
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
