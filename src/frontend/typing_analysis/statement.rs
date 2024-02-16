use crate::error::{Error, TerminationError};
use crate::frontend::typing_analysis::TypeAnalysis;
use crate::hir::statement::Statement;
use crate::symbol_table::SymbolTable;

impl<E> TypeAnalysis for Statement<E>
where
    E: TypeAnalysis,
{
    fn typing(
        &mut self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<(), TerminationError> {
        let Statement { id, expression, .. } = self;

        expression.typing(symbol_table, errors)?;
        let expression_type = expression.get_type().unwrap();
        symbol_table.set_type(id, expression_type.clone());
        Ok(())
    }
}
