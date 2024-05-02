use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::function::Function;
use crate::symbol_table::SymbolTable;

impl TypeAnalysis for Function {
    fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Function {
            id,
            statements,
            returned,
            location,
            ..
        } = self;

        // type all statements
        statements
            .iter_mut()
            .map(|statement| statement.typing(symbol_table, errors))
            .collect::<Vec<Result<(), TerminationError>>>()
            .into_iter()
            .collect::<Result<(), TerminationError>>()?;

        // type returned expression
        returned.typing(symbol_table, errors)?;

        // check returned type
        let expected_type = symbol_table.get_function_output_type(*id);
        returned
            .get_type()
            .unwrap()
            .eq_check(expected_type, location.clone(), errors)
    }
}
