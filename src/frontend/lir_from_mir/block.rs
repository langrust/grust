use crate::lir::block::Block as LIRBlock;
use crate::mir::block::Block;

use super::statement::lir_from_mir as statement_lir_from_mir;

/// Transform MIR block into LIR block.
pub fn lir_from_mir(block: Block) -> LIRBlock {
    let statements = block
        .statements
        .into_iter()
        .map(|statement| statement_lir_from_mir(statement))
        .collect();
    LIRBlock { statements }
}

#[cfg(test)]
mod lir_from_mir {
    use crate::common::constant::Constant;
    use crate::frontend::lir_from_mir::block::lir_from_mir;
    use crate::lir::block::Block as LIRBlock;
    use crate::lir::expression::Expression as LIRExpression;
    use crate::lir::pattern::Pattern as LIRPattern;
    use crate::lir::statement::r#let::Let;
    use crate::lir::statement::Statement as LIRStatement;
    use crate::mir::block::Block;
    use crate::mir::expression::Expression;
    use crate::mir::statement::Statement;

    #[test]
    fn should_create_lir_block_from_mir_block() {
        let block = Block {
            statements: vec![
                Statement::Let {
                    identifier: String::from("x"),
                    expression: Expression::Literal {
                        literal: Constant::Integer(1),
                    },
                },
                Statement::ExpressionLast {
                    expression: Expression::Identifier {
                        identifier: String::from("x"),
                    },
                },
            ],
        };
        let control = LIRBlock {
            statements: vec![
                LIRStatement::Let(Let {
                    pattern: LIRPattern::Identifier {
                        reference: false,
                        mutable: false,
                        identifier: String::from("x"),
                    },
                    expression: LIRExpression::Literal {
                        literal: Constant::Integer(1),
                    },
                }),
                LIRStatement::ExpressionLast(LIRExpression::Identifier {
                    identifier: String::from("x"),
                }),
            ],
        };
        assert_eq!(lir_from_mir(block), control)
    }
}
