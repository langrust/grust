use crate::ast::function::Function;
use crate::error::{Error, TerminationError};
use crate::frontend::hir_from_ast::{
    expression::hir_from_ast as expression_hir_from_ast,
    statement::hir_from_ast as statement_hir_from_ast,
};
use crate::hir::function::Function as HIRFunction;
use crate::symbol_table::SymbolTable;

/// Transform AST functions into HIR functions.
pub fn hir_from_ast(
    function: Function,
    symbol_table: &mut SymbolTable,
    errors: &mut Vec<Error>,
) -> Result<HIRFunction, TerminationError> {
    let Function {
        id,
        inputs,
        statements,
        returned: (_, returned),
        location,
    } = function;

    // create local context with all inputs
    symbol_table.local();
    let inputs = inputs
        .into_iter()
        .map(|(name, typing)| {
            let id = symbol_table.insert_identifier(name, true, location, errors)?;
            // TODO: add type to signal in symbol table
            Ok(id)
        })
        .collect::<Vec<Result<_, _>>>()
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    let id = symbol_table.get_function_id(&id, false, location, errors)?;

    let statements = statements
        .into_iter()
        .map(|statement| statement_hir_from_ast(statement, symbol_table, errors))
        .collect::<Vec<Result<_, _>>>()
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    let returned = expression_hir_from_ast(returned, symbol_table, errors)?;

    symbol_table.global();

    Ok(HIRFunction {
        id,
        inputs,
        statements,
        returned,
        location,
    })
}
