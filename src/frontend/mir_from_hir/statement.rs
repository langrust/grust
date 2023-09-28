use crate::{ast::statement::Statement, mir::statement::Statement as MIRStatement};

use super::expression::mir_from_hir as expression_mir_from_hir;

/// Transform HIR statement into MIR statement.
pub fn mir_from_hir(statement: Statement) -> MIRStatement {
    let Statement { id, expression, .. } = statement;
    MIRStatement::Let {
        identifier: id,
        expression: expression_mir_from_hir(expression),
    }
}
