use crate::lir::pattern::Pattern;
use crate::lir::statement::r#let::Let;
use crate::lir::statement::Statement as LIRStatement;
use crate::mir::statement::Statement;

use super::expression::lir_from_mir as expression_lir_from_mir;

/// Transform MIR statement into LIR statement.
pub fn lir_from_mir(statement: Statement) -> LIRStatement {
    match statement {
        Statement::Let {
            identifier,
            expression,
        } => LIRStatement::Let(Let {
            pattern: Pattern::Identifier {
                reference: false,
                mutable: false,
                identifier,
            },
            expression: expression_lir_from_mir(expression),
        }),
        Statement::LetTuple {
            identifiers,
            expression,
        } => {
            let elements = identifiers
                .into_iter()
                .map(|identifier| Pattern::Identifier {
                    reference: false,
                    mutable: false,
                    identifier,
                })
                .collect();
            LIRStatement::Let(Let {
                pattern: Pattern::Tuple { elements },
                expression: expression_lir_from_mir(expression),
            })
        }
        Statement::ExpressionLast { expression } => {
            LIRStatement::ExpressionLast(expression_lir_from_mir(expression))
        }
    }
}
