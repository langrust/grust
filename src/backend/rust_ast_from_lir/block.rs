use std::collections::BTreeSet;

use super::statement::rust_ast_from_lir as statement_rust_ast_from_lir;
use crate::lir::block::Block;

/// Transform LIR block into RustAST block.
pub fn rust_ast_from_lir(block: Block, crates: &mut BTreeSet<String>) -> syn::Block {
    let stmts = block
        .statements
        .into_iter()
        .map(|statement| statement_rust_ast_from_lir(statement, crates))
        .collect();
    syn::Block {
        stmts,
        brace_token: Default::default(),
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::backend::rust_ast_from_lir::block::rust_ast_from_lir;
    use crate::common::constant::Constant;
    use crate::lir::block::Block;
    use crate::lir::expression::Expression;
    use crate::lir::statement::Statement;
    use syn::*;

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

        let control: syn::Block = parse_quote!({
            let x = 1i64;
            x
        });

        assert_eq!(rust_ast_from_lir(block, &mut Default::default()), control)
    }
}
