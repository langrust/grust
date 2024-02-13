use crate::ast::equation::Equation;
use crate::error::{Error, TerminationError};
use crate::frontend::hir_from_ast::stream_expression::hir_from_ast as stream_expression_hir_from_ast;
use crate::hir::equation::Equation as HIREquation;
use crate::symbol_table::SymbolTable;

/// Transform AST equations into HIR equations.
pub fn hir_from_ast(
    equation: Equation,
    symbol_table: &mut SymbolTable,
    errors: &mut Vec<Error>,
) -> Result<HIREquation, TerminationError> {
    let Equation {
        scope,
        id,
        signal_type,
        expression,
        location,
    } = equation;

    let id = symbol_table.get_signal_id(&id, true, location, errors)?;

    Ok(HIREquation {
        id,
        expression: stream_expression_hir_from_ast(expression, symbol_table, errors)?,
        location,
    })
}
