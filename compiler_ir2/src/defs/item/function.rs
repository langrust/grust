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
    pub fn into_syn(self, crates: &mut BTreeSet<String>) -> syn::Item {
        let attributes = self.contract.into_syn(true);

        let inputs = self
            .inputs
            .into_iter()
            .map(|(name, typ)| {
                syn::FnArg::Typed(syn::PatType {
                    attrs: vec![],
                    pat: parse_quote!(#name),
                    colon_token: Default::default(),
                    ty: Box::new(typ.into_syn()),
                })
            })
            .collect();

        let sig = syn::Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: self.name.clone(),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs,
            variadic: None,
            output: syn::ReturnType::Type(Default::default(), Box::new(self.output.into_syn())),
        };

        let item_function = syn::Item::Fn(syn::ItemFn {
            attrs: attributes,
            vis: syn::Visibility::Public(Default::default()),
            sig,
            block: Box::new(self.body.into_syn(crates)),
        });

        item_function
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_rust_ast_function_from_ir2_function() {
        // use item::{Block, Function, Stmt};
        let function = Function {
            name: Loc::test_id("foo"),
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
        };

        let control = parse_quote! {
            pub fn foo(a: i64, b: i64) -> i64 {
                a + b
            }
        };
        assert_eq!(function.into_syn(&mut Default::default()), control)
    }
}
