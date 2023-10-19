use crate::{
    hir::{equation::Equation, stream_expression::StreamExpression},
    mir::statement::Statement,
};

use super::stream_expression::mir_from_hir as stream_expression_mir_from_hir;

/// Transform HIR equation into MIR statement.
pub fn mir_from_hir(equation: Equation) -> Statement {
    let Equation { id, expression, .. } = equation;
    match expression {
        StreamExpression::UnitaryNodeApplication { .. } => Statement::LetTuple {
            identifiers: vec![todo!("what is the node state's name?"), id],
            expression: stream_expression_mir_from_hir(expression),
        },
        _ => Statement::Let {
            identifier: id,
            expression: stream_expression_mir_from_hir(expression),
        },
    }
}
