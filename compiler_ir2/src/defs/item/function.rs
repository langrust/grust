//! [Function] module.

prelude! { Block }

/// A function definition.
#[derive(Debug, PartialEq)]
pub struct Function {
    /// The function's name.
    pub name: Ident,
    /// The inputs.
    pub inputs: Vec<(Ident, Typ)>,
    /// The output type.
    pub output: Typ,
    /// The body of the function.
    pub body: Block,
    /// The contract to prove.
    pub contract: Contract,
}

mk_new! { impl Function => new {
    name: impl Into<Ident> = name.into(),
    inputs: Vec<(Ident, Typ)>,
    output: Typ,
    body: Block,
    contract: Contract,
} }

impl Function {
    pub fn to_def_and_logic_tokens(&self, ctx: &ir0::Ctx) -> (TokenStream2, Option<TokenStream2>) {
        let inputs = self.inputs.iter().map(|(name, typ)| quote!( #name: #typ ));
        let name = &self.name;
        let output = &self.output;
        let pub_token = if ctx.conf.public {
            quote! {pub}
        } else {
            quote! {}
        };

        if ctx.conf.mode.greusot() {
            let logic_args = self.inputs.iter().map(|(name, ty)| {
                let mut ts = name.to_token_stream();
                if ty.needs_view() {
                    token![@].to_tokens(&mut ts)
                }
                ts
            });
            let logic_result = {
                let mut ts = Ident::result(name.span()).to_token_stream();
                if self.output.needs_view() {
                    token![@].to_tokens(&mut ts)
                }
                ts
            };
            let logic_inputs = self.inputs.iter().map(|(name, typ)| {
                let typ = typ.to_logic();
                quote!(#name : #typ)
            });
            let logic_output = self.output.to_logic();
            let logic_body = self.body.to_logic();
            let body = &self.body;
            let contract = self.contract.prepare_tokens(true);
            (
                quote! {
                    #contract
                    #[ensures(#logic_result == logical::#name(#(#logic_args),*))]
                    #pub_token fn #name(#(#inputs),*) -> #output
                    #body
                },
                Some(quote! {
                    #[open]
                    #[logic]
                    pub fn #name(#(#logic_inputs),*) -> #logic_output
                    #logic_body
                }),
            )
        } else {
            let body = &self.body;
            (
                quote! { #pub_token fn #name(#(#inputs),*) -> #output #body },
                None,
            )
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_rust_ast_function_from_ir2_function() {
        // use item::{Block, Function, Stmt};
        let (function, _) = Function {
            name: Loc::test_id("add"),
            inputs: vec![
                (Loc::test_id("a"), Typ::int()),
                (Loc::test_id("b"), Typ::int()),
            ],
            output: Typ::int(),
            body: Block {
                statements: vec![Stmt::ExprLast {
                    expr: Expr::binop(BOp::Add, Expr::test_ident("a"), Expr::test_ident("b")),
                }],
            },
            contract: Default::default(),
        }
        .to_def_and_logic_tokens(&ir0::Ctx::empty());

        let control = parse_quote! {
            pub fn add(a: i64, b: i64) -> i64 {
                a + b
            }
        };
        let function: syn::ItemFn = parse_quote!(#function);
        assert_eq!(function, control)
    }
}
