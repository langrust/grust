prelude! { Expr, Pattern }

/// A statement declaration.
#[derive(Debug, PartialEq)]
pub enum Stmt {
    /// A let-statement creating one variable: `let x = y + 1;`.
    Let {
        /// The variables created.
        pattern: Pattern,
        /// The expression associated to the variable.
        expr: Expr,
    },
    /// A returned expression.
    ExprLast {
        /// The returned expression.
        expr: Expr,
    },
}

mk_new! { impl Stmt =>
    Let: let_binding {
        pattern: Pattern,
        expr: Expr,
    }
    ExprLast: expression_last { expr: Expr }
}

impl Stmt {
    pub fn into_syn(self, crates: &mut BTreeSet<String>) -> syn::Stmt {
        match self {
            Self::Let { pattern, expr } => syn::Stmt::Local(syn::Local {
                attrs: vec![],
                let_token: Default::default(),
                pat: pattern.into_syn(),
                init: Some(syn::LocalInit {
                    eq_token: Default::default(),
                    expr: Box::new(expr.into_syn(crates)),
                    diverge: None,
                }),
                semi_token: Default::default(),
            }),
            Stmt::ExprLast { expr } => syn::Stmt::Expr(expr.into_syn(crates), None),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_rust_ast_let_statement_from_lir_let_statement() {
        let statement = Stmt::let_binding(
            Pattern::ident("x"),
            Expr::lit(Constant::int(parse_quote!(1i64))),
        );

        let control = parse_quote! {
            let x = 1i64;
        };
        assert_eq!(statement.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_let_statement_from_lir_let_statement_with_node_call() {
        let statement = Stmt::let_binding(
            Pattern::ident("o"),
            Expr::node_call(
                "node_state",
                "node",
                "NodeInput",
                vec![("i".into(), Expr::lit(Constant::int(parse_quote!(1i64))))],
            ),
        );

        let control = parse_quote! { let o = self.node_state.step(NodeInput { i: 1i64 }); };
        assert_eq!(statement.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_last_expression_from_lir_last_expression() {
        let statement = Stmt::expression_last(Expr::lit(Constant::int(parse_quote!(1i64))));

        let control = syn::Stmt::Expr(parse_quote! { 1i64 }, None);
        assert_eq!(statement.into_syn(&mut Default::default()), control)
    }
}
