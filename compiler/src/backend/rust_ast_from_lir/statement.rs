use std::collections::BTreeSet;

use super::{
    expression::rust_ast_from_lir as expression_rust_ast_from_lir,
    pattern::rust_ast_from_lir as pattern_rust_ast_from_lir,
};

prelude! {
    syn::*,
}

/// Transform LIR statement into RustAST statement.
pub fn rust_ast_from_lir(statement: lir::Stmt, crates: &mut BTreeSet<String>) -> Stmt {
    match statement {
        lir::Stmt::Let {
            pattern,
            expression,
        } => Stmt::Local(Local {
            attrs: vec![],
            let_token: Default::default(),
            pat: pattern_rust_ast_from_lir(pattern),
            init: Some(LocalInit {
                eq_token: Default::default(),
                expr: Box::new(expression_rust_ast_from_lir(expression, crates)),
                diverge: None,
            }),
            semi_token: Default::default(),
        }),
        lir::Stmt::ExprLast { expression } => {
            Stmt::Expr(expression_rust_ast_from_lir(expression, crates), None)
        }
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    prelude! {
        syn::*,
        backend::rust_ast_from_lir::statement::rust_ast_from_lir,
        lir::Pattern,
    }

    #[test]
    fn should_create_rust_ast_let_statement_from_lir_let_statement() {
        let statement = lir::Stmt::let_binding(
            Pattern::ident("x"),
            lir::Expr::lit(Constant::int(parse_quote!(1i64))),
        );

        let control = parse_quote! {
            let x = 1i64;
        };
        assert_eq!(
            rust_ast_from_lir(statement, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_let_statement_from_lir_let_statement_with_node_call() {
        let statement = lir::Stmt::let_binding(
            Pattern::ident("o"),
            lir::Expr::node_call(
                "node_state",
                "node",
                "NodeInput",
                vec![(
                    "i".into(),
                    lir::Expr::lit(Constant::int(parse_quote!(1i64))),
                )],
            ),
        );

        let control = parse_quote! { let o = self.node_state.step(NodeInput { i: 1i64 }); };
        assert_eq!(
            rust_ast_from_lir(statement, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_last_expression_from_lir_last_expression() {
        let statement =
            lir::Stmt::expression_last(lir::Expr::lit(Constant::int(parse_quote!(1i64))));

        let control = Stmt::Expr(parse_quote! { 1i64 }, None);
        assert_eq!(
            rust_ast_from_lir(statement, &mut Default::default()),
            control
        )
    }
}
