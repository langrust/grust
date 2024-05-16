use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::statement::Statement;
use crate::symbol_table::SymbolTable;

impl<E> TypeAnalysis for Statement<E>
where
    E: TypeAnalysis,
{
    // precondition: identifiers associated with statement is already typed
    // postcondition: expression associated with statement is typed and checked
    fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Statement {
            pattern,
            expression,
            location,
        } = self;

        let expected_type = pattern.construct_statement_type(symbol_table, errors)?;
        expression.typing(symbol_table, errors)?;
        
        let expression_type = expression.get_type().unwrap();
        expression_type.eq_check(&expected_type, location.clone(), errors)?;

        Ok(())
    }
}
