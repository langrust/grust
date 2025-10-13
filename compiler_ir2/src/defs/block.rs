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

impl ToTokens for Block {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let stmts = self.statements.iter();
        tokens.extend(quote!({ #(#stmts)* }))
    }
}
impl ToLogicTokens for Block {
    fn to_logic_tokens(&self, tokens: &mut TokenStream2) {
        let stmts = self.statements.iter().map(|stmt| stmt.to_logic());
        tokens.extend(quote!({ #(#stmts)* }))
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
            Stmt::expr_last(Expr::test_ident("x")),
        ]);

        let control: syn::Block = parse_quote!({
            let x = 1i64;
            x
        });

        let blk: syn::Block = parse_quote!(#block);
        assert_eq!(blk, control)
    }
}
