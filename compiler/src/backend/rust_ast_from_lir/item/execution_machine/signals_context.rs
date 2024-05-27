use crate::backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir;
use crate::lir::item::execution_machine::signals_context::SignalsContext;
use proc_macro2::Span;
use quote::format_ident;
use syn::*;

/// Transform LIR run-loop into items.
pub fn rust_ast_from_lir(signals_context: SignalsContext) -> Vec<Item> {
    let SignalsContext { elements } = signals_context;

    let context_struct = {
        let fields = elements.iter().map(|(element_name, element_ty)| {
            let name = Ident::new(element_name, Span::call_site());
            let ty = type_rust_ast_from_lir(element_ty.clone());
            Field {
                attrs: vec![],
                vis: Visibility::Public(Default::default()),
                ident: Some(name),
                colon_token: Default::default(),
                ty,
                mutability: FieldMutability::None,
            }
        });
        let name = Ident::new("Context", Span::call_site());
        parse_quote! { #[derive(Clone, Copy, Debug, PartialEq, Default)] pub struct #name { #(#fields),* } }
    };

    let mut impl_items = vec![ImplItem::Fn(ImplItemFn {
        attrs: Default::default(),
        vis: Visibility::Inherited,
        defaultness: None,
        sig: Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: Ident::new("init", Span::call_site()),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Default::default(),
            variadic: None,
            output: {
                let identifier = Ident::new("Context", Span::call_site());
                ReturnType::Type(Default::default(), Box::new(parse_quote!(#identifier)))
            },
        },
        block: Block {
            brace_token: Default::default(),
            stmts: vec![Stmt::Expr(
                Expr::Call(ExprCall {
                    attrs: vec![],
                    func: Box::new(Expr::Path(ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: parse_quote!(Default::default),
                    })),
                    paren_token: Default::default(),
                    args: Default::default(),
                }),
                None,
            )],
        },
    })];
    let context_impl = Item::Impl(ItemImpl {
        attrs: Default::default(),
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics: Default::default(),
        trait_: None,
        self_ty: Box::new(parse_quote!(Context)),
        brace_token: Default::default(),
        items: impl_items,
    });

    vec![context_struct, context_impl]
}
