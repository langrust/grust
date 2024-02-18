use crate::ast::equation::Equation;
use crate::error::{Error, TerminationError};
use crate::hir::{
    statement::Statement as HIRStatement,
    stream_expression::StreamExpression as HIRStreamExpression,
};
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for Equation {
    type HIR = HIRStatement<HIRStreamExpression>;

    // precondition: equation's signal is already stored in symbol table
    // postcondition: construct HIR equation and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let Equation {
            id,
            expression,
            location,
            ..
        } = self;

        let id = symbol_table.get_signal_id(&id, true, location.clone(), errors)?;

        Ok(HIRStatement {
            id,
            expression: expression.hir_from_ast(symbol_table, errors)?,
            location,
        })
    }
}
