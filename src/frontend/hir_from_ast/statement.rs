use crate::ast::statement::Statement;
use crate::error::{Error, TerminationError};
use crate::frontend::hir_from_ast::expression::hir_from_ast as expression_hir_from_ast;
use crate::hir::statement::Statement as HIRStatement;
use crate::symbol_table::SymbolTable;

/// Transform AST statements into HIR statements.
pub fn hir_from_ast(
    statement: Statement,
    symbol_table: &mut SymbolTable,
    errors: &mut Vec<Error>,
) -> Result<HIRStatement, TerminationError> {
    let Statement {
        id,
        element_type,
        expression,
        location,
    } = statement;

    let id = symbol_table.insert_identifier(id, None, true, location, errors)?;

    Ok(HIRStatement {
        id,
        expression: expression_hir_from_ast(expression, symbol_table, errors)?,
        location,
    })
}
