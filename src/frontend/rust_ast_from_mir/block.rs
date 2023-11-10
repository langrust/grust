use crate::rust_ast::block::Block as RustASTBlock;
use crate::mir::block::Block;

use super::statement::rust_ast_from_mir as statement_rust_ast_from_mir;

/// Transform MIR block into RustAST block.
pub fn rust_ast_from_mir(block: Block) -> RustASTBlock {
    let statements = block
        .statements
        .into_iter()
        .map(statement_rust_ast_from_mir)
        .collect();
    RustASTBlock { statements }
}

#[cfg(test)]
mod rust_ast_from_mir {
    use crate::common::constant::Constant;
    use crate::frontend::rust_ast_from_mir::block::rust_ast_from_mir;
    use crate::rust_ast::block::Block as RustASTBlock;
    use crate::rust_ast::expression::Expression as RustASTExpression;
    use crate::rust_ast::pattern::Pattern as RustASTPattern;
    use crate::rust_ast::statement::r#let::Let;
    use crate::rust_ast::statement::Statement as RustASTStatement;
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
        let control = RustASTBlock {
            statements: vec![
                RustASTStatement::Let(Let {
                    pattern: RustASTPattern::Identifier {
                        reference: false,
                        mutable: false,
                        identifier: String::from("x"),
                    },
                    expression: RustASTExpression::Literal {
                        literal: Constant::Integer(1),
                    },
                }),
                RustASTStatement::ExpressionLast(RustASTExpression::Identifier {
                    identifier: String::from("x"),
                }),
            ],
        };
        assert_eq!(rust_ast_from_mir(block), control)
    }
}
