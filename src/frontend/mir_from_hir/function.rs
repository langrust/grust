use crate::{
    ast::function::Function,
    mir::{block::Block, item::function::Function as MIRFunction, statement::Statement},
};

use super::{
    expression::mir_from_hir as expression_mir_from_hir,
    statement::mir_from_hir as statement_mir_from_hir,
};
/// Transform HIR function into MIR function.
pub fn mir_from_hir(function: Function) -> MIRFunction {
    let Function {
        id,
        inputs,
        statements,
        returned: (output, last_expression),
        ..
    } = function;

    let mut statements = statements
        .into_iter()
        .map(|statement| statement_mir_from_hir(statement))
        .collect::<Vec<_>>();
    statements.push(Statement::ExpressionLast {
        expression: expression_mir_from_hir(last_expression),
    });

    MIRFunction {
        name: id,
        inputs,
        output,
        body: Block { statements },
    }
}
