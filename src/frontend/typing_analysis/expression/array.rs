use std::collections::HashMap;

use crate::hir::{expression::Expression, typedef::Typedef};
use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the array expression.
    pub fn typing_array(
        &mut self,
        symbol_table: &mut SymbolTable,
        user_types_context: &HashMap<String, Typedef>,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self {
            // an array is composed of `n` elements of the same type `t` and
            // its type is `[t; n]`
            Expression::Array {
                elements,
                typing,
                location,
            } => {
                if elements.len() == 0 {
                    let error = Error::ExpectInput {
                        location: location.clone(),
                    };
                    errors.push(error);
                    return Err(TerminationError);
                }

                elements
                    .iter_mut()
                    .map(|element| {
                        element.typing(symbol_table, user_types_context, errors)
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                let first_type = elements[0].get_type().unwrap(); // todo: manage zero element error
                elements
                    .iter()
                    .map(|element| {
                        let element_type = element.get_type().unwrap();
                        element_type.eq_check(first_type, location.clone(), errors)
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                let array_type = Type::Array(Box::new(first_type.clone()), elements.len());

                *typing = Some(array_type);
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
