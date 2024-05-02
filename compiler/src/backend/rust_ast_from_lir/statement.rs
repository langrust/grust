use std::collections::BTreeSet;

use super::expression::rust_ast_from_lir as expression_rust_ast_from_lir;
use crate::lir::statement::Statement;
use proc_macro2::Span;
use syn::*;

/// Transform LIR statement into RustAST statement.
pub fn rust_ast_from_lir(statement: Statement, crates: &mut BTreeSet<String>) -> Stmt {
    match statement {
        Statement::Let {
            identifier,
            expression,
        } => Stmt::Local(Local {
            attrs: vec![],
            let_token: Default::default(),
            pat: Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: Ident::new(&identifier, Span::call_site()),
                subpat: None,
            }),
            init: Some(LocalInit {
                eq_token: Default::default(),
                expr: Box::new(expression_rust_ast_from_lir(expression, crates)),
                diverge: None,
            }),
            semi_token: Default::default(),
        }),
        Statement::ExpressionLast { expression } => {
            Stmt::Expr(expression_rust_ast_from_lir(expression, crates), None)
        }
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::backend::rust_ast_from_lir::statement::rust_ast_from_lir;
    use crate::common::constant::Constant;
    use crate::lir::expression::Expression;
    use crate::lir::statement::Statement;
    use syn::*;
    #[test]
    fn should_create_rust_ast_let_statement_from_lir_let_statement() {
        let statement = Statement::Let {
            identifier: String::from("x"),
            expression: Expression::Literal {
                literal: Constant::Integer(parse_quote!(1)),
            },
        };

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
        let statement = Statement::Let {
            identifier: String::from("o"),
            expression: Expression::NodeCall {
                node_identifier: String::from("node_state"),
                input_name: String::from("NodeInput"),
                input_fields: vec![(
                    String::from("i"),
                    Expression::Literal {
                        literal: Constant::Integer(parse_quote!(1)),
                    },
                )],
            },
        };

        let control = parse_quote! { let o = self.node_state.step(NodeInput { i: 1i64 }); };
        assert_eq!(
            rust_ast_from_lir(statement, &mut Default::default()),
            control
        )
    }

    #[test]
    fn should_create_rust_ast_last_expression_from_lir_last_expression() {
        let statement = Statement::ExpressionLast {
            expression: Expression::Literal {
                literal: Constant::Integer(parse_quote!(1)),
            },
        };

        let control = Stmt::Expr(parse_quote! { 1i64 }, None);
        assert_eq!(
            rust_ast_from_lir(statement, &mut Default::default()),
            control
        )
    }
}
