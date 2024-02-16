use crate::common::{location::Location, r#type::Type};
use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::expression::ExpressionKind;
use crate::symbol_table::SymbolTable;

impl<E> ExpressionKind<E>
where
    E: TypeAnalysis,
{
    /// Add a [Type] to the sort expression.
    pub fn typing_sort(
        &mut self,
        location: &Location,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Type, TerminationError> {
        match self {
            ExpressionKind::Sort {
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

                        // check it is a sorting function: (element_type, element_type) -> int
                        function_type.eq_check(
                            &Type::Abstract(
                                vec![*element_type.clone(), *element_type.clone()],
                                Box::new(Type::Integer),
                            ),
                            location.clone(),
                            errors,
                        )?;

                        Ok(Type::Array(element_type.clone(), *size))
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
