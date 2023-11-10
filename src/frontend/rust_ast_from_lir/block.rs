use crate::rust_ast::block::Block as RustASTBlock;
use crate::lir::block::Block;

use super::statement::rust_ast_from_lir as statement_rust_ast_from_lir;

/// Transform LIR block into RustAST block.
pub fn rust_ast_from_lir(block: Block) -> RustASTBlock {
    let statements = block
        .statements
        .into_iter()
        .map(statement_rust_ast_from_lir)
        .collect();
    RustASTBlock { statements }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::common::constant::Constant;
    use crate::frontend::rust_ast_from_lir::block::rust_ast_from_lir;
    use crate::rust_ast::block::Block as RustASTBlock;
    use crate::rust_ast::expression::Expression as RustASTExpression;
    use crate::rust_ast::pattern::Pattern as RustASTPattern;
    use crate::rust_ast::statement::r#let::Let;
    use crate::rust_ast::statement::Statement as RustASTStatement;
    use crate::lir::block::Block;
    use crate::lir::expression::Expression;
    use crate::lir::statement::Statement;

    #[test]
    fn should_create_rust_ast_block_from_lir_block() {
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
        assert_eq!(rust_ast_from_lir(block), control)
    }
}
