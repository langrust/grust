prelude! { Expr, Pattern }

/// A statement declaration.
#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    /// A let-statement creating one variable: `let x = y + 1;`.
    Let {
        /// The variables created.
        pattern: Pattern,
        /// The expression associated to the variable.
        expr: Expr,
    },
    /// Log statement.
    Log {
        /// Identifier to log.
        ident: syn::Ident,
        /// Expression to access identifier.
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
    Log: log { ident: syn::Ident, expr: Expr }
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
            Self::ExprLast { expr } => syn::Stmt::Expr(expr.into_syn(crates), None),
            Self::Log { ident, expr } => {
                let lit = syn::LitStr::new(&format!("{ident}: {{:?}}"), ident.span());
                let expr = expr.into_syn(crates);
                parse_quote! { println!(#lit, #expr); }
            }
        }
    }
    pub fn into_logic(self, crates: &mut BTreeSet<String>) -> syn::Stmt {
        match self {
            Self::Let { pattern, expr } => syn::Stmt::Local(syn::Local {
                attrs: vec![],
                let_token: Default::default(),
                pat: pattern.into_syn(),
                init: Some(syn::LocalInit {
                    eq_token: Default::default(),
                    expr: Box::new(expr.into_logic(crates)),
                    diverge: None,
                }),
                semi_token: Default::default(),
            }),
            Self::ExprLast { expr } => syn::Stmt::Expr(expr.into_logic(crates), None),
            Self::Log { ident, expr } => {
                let lit = syn::LitStr::new(&format!("{ident}: {{:?}}"), ident.span());
                let expr = expr.into_syn(crates);
                parse_quote! { println!(#lit, #expr); }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_rust_ast_let_statement_from_ir2_let_statement() {
        let statement = Stmt::let_binding(
            Pattern::test_ident("x"),
            Expr::lit(Constant::int(parse_quote!(1i64))),
        );

        let control = parse_quote! {
            let x = 1i64;
        };
        assert_eq!(statement.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_let_statement_from_ir2_let_statement_with_node_call() {
        let statement = Stmt::let_binding(
            Pattern::test_ident("o"),
            Expr::node_call(
                Loc::test_id("node_state"),
                Loc::test_id("node"),
                Loc::test_id("NodeInput"),
                vec![(
                    Loc::test_id("i"),
                    Expr::lit(Constant::int(parse_quote!(1i64))),
                )],
                None,
            ),
        );

        let control = parse_quote! { let o = self.node_state.step(NodeInput { i: 1i64 }); };
        assert_eq!(statement.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_last_expression_from_ir2_last_expression() {
        let statement = Stmt::expression_last(Expr::lit(Constant::int(parse_quote!(1i64))));

        let control = syn::Stmt::Expr(parse_quote! { 1i64 }, None);
        assert_eq!(statement.into_syn(&mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_println_from_ir2_log() {
        let statement = Stmt::log(
            parse_quote!(x),
            Expr::InputAccess {
                identifier: parse_quote!(x),
            },
        );

        let control = parse_quote! { println!("x: {:?}", input.x); };
        assert_eq!(statement.into_syn(&mut Default::default()), control)
    }
}
