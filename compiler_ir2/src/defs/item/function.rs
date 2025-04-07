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
    pub fn into_syn(
        self,
        ctx: &ir0::Ctx,
        crates: &mut BTreeSet<String>,
    ) -> (syn::Item, Option<syn::Item>) {
        let inputs = self.inputs.iter().map(|(name, typ)| {
            syn::FnArg::Typed(syn::PatType {
                attrs: vec![],
                pat: parse_quote!(#name),
                colon_token: Default::default(),
                ty: Box::new(typ.into_syn()),
            })
        });
        let name = self.name;
        let output = self.output.into_syn();

        if ctx.conf.greusot {
            let logic_args = self.inputs.iter().map(|(name, ty)| {
                let mut ts = name.to_token_stream();
                if ty.needs_view() {
                    ts.extend(syn::token::At::default().to_token_stream());
                }
                ts
            });
            let logic_result = {
                let mut ts = Ident::new("result", name.span()).to_token_stream();
                if self.output.needs_view() {
                    ts.extend(syn::token::At::default().to_token_stream());
                }
                ts
            };
            let logic_inputs = self.inputs.iter().map(|(name, typ)| {
                syn::FnArg::Typed(syn::PatType {
                    attrs: vec![],
                    pat: parse_quote!(#name),
                    colon_token: Default::default(),
                    ty: Box::new(typ.into_logic()),
                })
            });
            let logic_output = self.output.into_logic();
            let logic_body = self.body.clone().into_logic(crates);
            let body = self.body.into_syn(crates);
            let attributes = self.contract.into_syn(true);
            (
                syn::Item::Fn(parse_quote! {
                    #(#attributes)*
                    #[ensures(#logic_result == logical::#name(#(#logic_args),*))]
                    pub fn #name(#(#inputs),*) -> #output
                    #body
                }),
                Some(parse_quote! {
                    #[open]
                    #[logic]
                    pub fn #name(#(#logic_inputs),*) -> #logic_output
                    #logic_body
                }),
            )
        } else {
            let body = self.body.into_syn(crates);
            (
                syn::Item::Fn(parse_quote! {
                    pub fn #name(#(#inputs),*) -> #output
                    #body
                }),
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
        .into_syn(&ir0::Ctx::empty(), &mut Default::default());

        let control = parse_quote! {
            pub fn add(a: i64, b: i64) -> i64 {
                a + b
            }
        };
        assert_eq!(function, control)
    }
}
