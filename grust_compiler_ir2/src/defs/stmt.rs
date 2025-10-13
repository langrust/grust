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
    ExprLast: expr_last { expr: Expr }
    Log: log { ident: syn::Ident, expr: Expr }
}

impl ToTokens for Stmt {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Let { pattern, expr } => tokens.extend(quote! {
                let #pattern = #expr;
            }),
            Self::ExprLast { expr } => expr.to_tokens(tokens),
            Self::Log { ident, expr } => {
                let str = format!("{}: {{:?}}", ident);
                tokens.extend(quote! {
                    println!(#str, #expr);
                })
            }
        }
    }
}
impl ToLogicTokens for Stmt {
    fn to_logic_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Let { pattern, expr } => {
                let expr = expr.to_logic();
                tokens.extend(quote!(let #pattern = #expr;))
            }
            Self::ExprLast { expr } => expr.to_logic_tokens(tokens),
            Self::Log { ident, expr } => {
                let str = format!("{}: {{:?}}", ident);
                tokens.extend(quote!(println!(#str, #expr);))
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
        let stmt: syn::Stmt = parse_quote!(#statement);
        assert_eq!(stmt, control)
    }

    #[test]
    fn should_create_rust_ast_let_statement_from_ir2_let_statement_with_comp_call() {
        let statement = Stmt::let_binding(
            Pattern::test_ident("o"),
            Expr::comp_call(
                Loc::test_id("comp_state"),
                Loc::test_id("component"),
                vec![(
                    Loc::test_id("i"),
                    Expr::lit(Constant::int(parse_quote!(1i64))),
                )],
                std::iter::once(Loc::test_id("out")),
                None,
            ),
        );

        let control = parse_quote! { let o = {
            let ComponentOutput {out} = <ComponentState as grust::core::Component>::step(&mut self.comp_state, ComponentInput { i : 1i64 });
            (out)
        }; };
        let stmt: syn::Stmt = parse_quote!(#statement);
        assert_eq!(stmt, control)
    }

    #[test]
    fn should_create_rust_ast_last_expression_from_ir2_last_expression() {
        let statement = Stmt::expr_last(Expr::lit(Constant::int(parse_quote!(1i64))));

        let control = syn::Stmt::Expr(parse_quote! { 1i64 }, None);
        let stmt = syn::Stmt::Expr(parse_quote!(#statement), None);
        assert_eq!(stmt, control)
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
        let stmt: syn::Stmt = parse_quote!(#statement);
        assert_eq!(stmt, control)
    }
}
