prelude! {}

/// A block declaration.
#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    /// The block's statements.
    pub statements: Vec<Stmt>,
}

mk_new! { impl Block => new {
    statements: Vec<Stmt>,
}}

impl Block {
    pub fn into_syn(self, crates: &mut BTreeSet<String>) -> syn::Block {
        let stmts = self
            .statements
            .into_iter()
            .map(|statement| statement.into_syn(crates))
            .collect();
        syn::Block {
            stmts,
            brace_token: Default::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_rust_ast_block_from_ir2_block() {
        let block = Block::new(vec![
            Stmt::let_binding(
                Pattern::test_ident("x"),
                Expr::lit(Constant::int(parse_quote!(1i64))),
            ),
            Stmt::expression_last(Expr::test_ident("x")),
        ]);

        let control: syn::Block = parse_quote!({
            let x = 1i64;
            x
        });

        assert_eq!(block.into_syn(&mut Default::default()), control)
    }
}
