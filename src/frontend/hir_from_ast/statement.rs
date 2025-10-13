use crate::ast::statement::Statement;
use crate::error::{Error, TerminationError};
use crate::hir::{expression::Expression as HIRExpression, statement::Statement as HIRStatement};
use crate::symbol_table::SymbolTable;

use super::HIRFromAST;

impl HIRFromAST for Statement {
    type HIR = HIRStatement<HIRExpression>;

    // precondition: NOTHING is in symbol table
    // postcondition: construct HIR statement and check identifiers good use
    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let Statement {
            id,
            element_type,
            expression,
            location,
        } = self;

        let typing = element_type.hir_from_ast(&location, symbol_table, errors)?;
        let id =
            symbol_table.insert_identifier(id, Some(typing), true, location.clone(), errors)?;

        Ok(HIRStatement {
            id,
            expression: expression.hir_from_ast(symbol_table, errors)?,
            location,
        })
    }
}
