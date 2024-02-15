use crate::{
    hir::statement::Statement, lir::statement::Statement as LIRStatement, symbol_table::SymbolTable,
};

use super::expression::lir_from_hir as expression_lir_from_hir;

/// Transform HIR statement into LIR statement.
pub fn lir_from_hir(statement: Statement, symbol_table: &SymbolTable) -> LIRStatement {
    let Statement { id, expression, .. } = statement;
    LIRStatement::Let {
        identifier: symbol_table.get_name(&id).clone(),
        expression: expression_lir_from_hir(expression, symbol_table),
    }
}
