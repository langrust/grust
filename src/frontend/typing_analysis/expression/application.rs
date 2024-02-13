use std::collections::HashMap;

use crate::error::{Error, TerminationError};
use crate::hir::{expression::Expression, typedef::Typedef};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the application expression.
    pub fn typing_application(
        &mut self,
        symbol_table: &mut SymbolTable,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // an application expression type is the result of the application
            // of the inputs types to the abstraction/function type
            Expression::Application {
                function_expression,
                inputs,
                typing,
                location,
            } => {
                // type all inputs
                inputs
                    .iter_mut()
                    .map(|input| input.typing(symbol_table, user_types_context, errors))
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                let input_types = inputs
                    .iter()
                    .map(|input| input.get_type().unwrap().clone())
                    .collect::<Vec<_>>();

                // type the function expression
                function_expression.typing(symbol_table, user_types_context, errors)?;

                // compute the application type
                let application_type = function_expression.get_type_mut().unwrap().apply(
                    input_types,
                    location.clone(),
                    errors,
                )?;

                *typing = Some(application_type);
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
