prelude! {
    backend::rust_ast_from_lir::expression::rust_ast_from_lir as expression_rust_ast_from_lir,
    lir::item::state_machine::state::init::{Init, StateElementInit},
    macro2::Span, syn::*, quote::format_ident,
}

/// Transform LIR init into RustAST implementation method.
pub fn rust_ast_from_lir(init: Init, crates: &mut BTreeSet<String>) -> ImplItemFn {
    let state_ty = Ident::new(
        &to_camel_case(&format!("{}State", init.node_name)),
        Span::call_site(),
    );
    let signature = syn::Signature {
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
        output: ReturnType::Type(Default::default(), parse_quote! { #state_ty }),
    };

    let fields = init
        .state_elements_init
        .into_iter()
        .map(|element| -> FieldValue {
            match element {
                StateElementInit::BufferInit {
                    identifier,
                    initial_expression,
                } => {
                    let id = format_ident!("{}", identifier);
                    let expr: Expr = expression_rust_ast_from_lir(initial_expression, crates);
                    parse_quote! { #id : #expr }
                }
                StateElementInit::CalledNodeInit {
                    identifier,
                    node_name,
                } => {
                    let id = Ident::new(&identifier, Span::call_site());
                    let called_state_ty = Ident::new(
                        &to_camel_case(&format!("{}State", node_name)),
                        Span::call_site(),
                    );
                    parse_quote! { #id : #called_state_ty::init () }
                }
            }
        })
        .collect();

    let body = syn::Block {
        brace_token: Default::default(),
        stmts: vec![Stmt::Expr(
            Expr::Struct(ExprStruct {
                attrs: vec![],
                path: parse_quote! { #state_ty },
                brace_token: Default::default(),
                dot2_token: None,
                rest: None,
                fields,
                qself: None, // Add the qself field here
            }),
            None,
        )],
    };
    ImplItemFn {
        attrs: vec![],
        vis: Visibility::Public(Default::default()),
        defaultness: None,
        sig: signature,
        block: body,
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    prelude! {
        syn::*,
        backend::rust_ast_from_lir::item::state_machine::state::init::rust_ast_from_lir,
        lir::{
            item::state_machine::state::init::{Init, StateElementInit},
        },
    }

    #[test]
    fn should_create_rust_ast_associated_method_from_lir_node_init() {
        let init = Init {
            invariant_initialisation: vec![],
            node_name: format!("Node"),
            state_elements_init: vec![
                StateElementInit::BufferInit {
                    identifier: format!("mem_i"),
                    initial_expression: lir::Expr::lit(Constant::int(parse_quote!(0i64))),
                },
                StateElementInit::CalledNodeInit {
                    identifier: format!("called_node_state"),
                    node_name: format!("CalledNode"),
                },
            ],
        };

        let control = parse_quote! {
            pub fn init() -> NodeState {
                NodeState {
                    mem_i: 0i64,
                    called_node_state: CalledNodeState::init()
                }
            }
        };
        assert_eq!(rust_ast_from_lir(init, &mut Default::default()), control)
    }
}
