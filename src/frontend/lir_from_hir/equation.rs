use crate::{hir::equation::Equation, lir::statement::Statement, symbol_table::SymbolTable};

use super::stream_expression::lir_from_hir as stream_expression_lir_from_hir;

/// Transform HIR equation into LIR statement.
pub fn lir_from_hir(equation: Equation, symbol_table: &SymbolTable) -> Statement {
    let Equation { id, expression, .. } = equation;
    Statement::Let {
        identifier: symbol_table.get_name(&id).clone(),
        expression: stream_expression_lir_from_hir(expression, symbol_table),
    }
}
