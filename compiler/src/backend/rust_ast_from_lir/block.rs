use std::collections::BTreeSet;

prelude! {
    lir::Block,
}

use super::statement::rust_ast_from_lir as statement_rust_ast_from_lir;

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
    prelude! {
        backend::rust_ast_from_lir::block::rust_ast_from_lir,
        lir::{ Block, Pattern, Stmt },
    }

    #[test]
    fn should_create_rust_ast_block_from_lir_block() {
        let block = Block {
            statements: vec![
                Stmt::let_binding(
                    Pattern::ident("x"),
                    lir::Expr::lit(Constant::int(parse_quote!(1i64))),
                ),
                Stmt::expression_last(lir::Expr::ident("x")),
            ],
        };

        let control: syn::Block = parse_quote!({
            let x = 1i64;
            x
        });

        assert_eq!(rust_ast_from_lir(block, &mut Default::default()), control)
    }
}
