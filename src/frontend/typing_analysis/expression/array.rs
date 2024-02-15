use crate::common::r#type::Type;
use crate::error::{Error, TerminationError};
use crate::hir::expression::{Expression, ExpressionKind};
use crate::symbol_table::SymbolTable;

impl Expression {
    /// Add a [Type] to the array expression.
    pub fn typing_array(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        match self.kind {
            // an array is composed of `n` elements of the same type `t` and
            // its type is `[t; n]`
            ExpressionKind::Array { ref mut elements } => {
                if elements.len() == 0 {
                    let error = Error::ExpectInput {
                        location: self.location.clone(),
                    };
                    errors.push(error);
                    return Err(TerminationError);
                }

                elements
                    .iter_mut()
                    .map(|element| element.typing(symbol_table, errors))
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                let first_type = elements[0].get_type().unwrap(); // todo: manage zero element error
                elements
                    .iter()
                    .map(|element| {
                        let element_type = element.get_type().unwrap();
                        element_type.eq_check(first_type, self.location.clone(), errors)
                    })
                    .collect::<Vec<Result<(), TerminationError>>>()
                    .into_iter()
                    .collect::<Result<(), TerminationError>>()?;

                let array_type = Type::Array(Box::new(first_type.clone()), elements.len());

                self.typing = Some(array_type);
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
