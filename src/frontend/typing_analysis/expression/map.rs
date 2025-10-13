use crate::common::{location::Location, r#type::Type};
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::expression::ExpressionKind;
use crate::symbol_table::SymbolTable;

impl<E> ExpressionKind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Type] to the map expression.
    pub fn typing_map(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Type, TerminationError> {
        match self {
            ExpressionKind::Map {
                ref mut expression,
                ref mut function_expression,
            } => {
                // type the expression
                expression.typing(symbol_table, errors)?;

                // verify it is an array
                match expression.get_type().unwrap() {
                    Type::Array(element_type, size) => {
                        // type the function expression
                        function_expression.typing(symbol_table, errors)?;
                        let function_type = function_expression.get_type_mut().unwrap();

                        // apply the function type to the type of array's elements
                        let new_element_type = function_type.apply(
                            vec![*element_type.clone()],
                            location.clone(),
                            errors,
                        )?;

                        Ok(Type::Array(Box::new(new_element_type), *size))
                    }
                    given_type => {
                        let error = Error::ExpectArray {
                            given_type: given_type.clone(),
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
