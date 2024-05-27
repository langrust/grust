use crate::backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir;
use crate::common::convert_case::camel_case;
use crate::lir::item::execution_machine::signals_context::SignalsContext;
use proc_macro2::Span;
use quote::format_ident;
use syn::*;

/// Transform LIR run-loop into items.
pub fn rust_ast_from_lir(signals_context: SignalsContext) -> Vec<Item> {
    let SignalsContext {
        elements,
        components,
    } = signals_context;

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
    // todo: for all components, create its input generator
    components
        .into_iter()
        .for_each(|(component_name, input_fields)| {
            let input_getter =
                Ident::new(&format!("get_{component_name}_inputs"), Span::call_site());
            let component_input_name =
                format_ident!("{}", camel_case(&format!("{component_name}Input")));

            let input_fields: Vec<FieldValue> = input_fields
                .into_iter()
                .map(|(field_name, in_context)| {
                    let field_id = Ident::new(&field_name, Span::call_site());
                    let in_context_id = Ident::new(&in_context, Span::call_site());
                    let expr: Expr = parse_quote!(self.#in_context_id);
                    parse_quote! { #field_id : #expr }
                })
                .collect();
            let result: ExprStruct = parse_quote! { #component_input_name { #(#input_fields),* }};

            let function = ImplItem::Fn(ImplItemFn {
                attrs: Default::default(),
                vis: Visibility::Inherited,
                defaultness: None,
                sig: Signature {
                    constness: None,
                    asyncness: None,
                    unsafety: None,
                    abi: None,
                    fn_token: Default::default(),
                    ident: input_getter,
                    generics: Default::default(),
                    paren_token: Default::default(),
                    inputs: vec![FnArg::Receiver(Receiver {
                        attrs: vec![],
                        reference: Some((Default::default(), None)),
                        mutability: None,
                        self_token: Default::default(),
                        colon_token: Default::default(),
                        ty: Box::new(parse_quote!(Self)),
                    })]
                    .into_iter()
                    .collect(),
                    variadic: None,
                    output: {
                        ReturnType::Type(
                            Default::default(),
                            Box::new(parse_quote!(#component_input_name)),
                        )
                    },
                },
                block: Block {
                    brace_token: Default::default(),
                    stmts: vec![Stmt::Expr(Expr::Struct(result), None)],
                },
            });

            impl_items.push(function)
        });

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
