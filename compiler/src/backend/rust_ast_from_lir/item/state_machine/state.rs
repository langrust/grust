use std::collections::BTreeSet;

prelude! {
    backend::{
        rust_ast_from_lir::item::state_machine::state::init::rust_ast_from_lir
            as init_rust_ast_from_lir,
        rust_ast_from_lir::item::state_machine::state::step::rust_ast_from_lir
            as step_rust_ast_from_lir,
        rust_ast_from_lir::typ::rust_ast_from_lir
            as type_rust_ast_from_lir,
    },
    lir::item::state_machine::state::{State, StateElement},
    quote::format_ident,
}

/// RustAST init method construction from LIR init.
pub mod init;
/// RustAST step method construction from LIR step.
pub mod step;

/// Transform LIR state into RustAST structure and implementation.
pub fn rust_ast_from_lir(
    state: State,
    crates: &mut BTreeSet<String>,
) -> (syn::ItemStruct, syn::ItemImpl) {
    let fields: Vec<syn::Field> = state
        .elements
        .into_iter()
        .map(|element| match element {
            StateElement::Buffer { identifier, typ } => {
                let identifier = format_ident!("{identifier}");
                let ty = type_rust_ast_from_lir(typ);
                parse_quote! { #identifier : #ty }
            }
            StateElement::CalledNode {
                identifier,
                node_name,
            } => {
                let name = format_ident!("{}", to_camel_case(&format!("{}State", node_name)));
                let identifier = format_ident!("{identifier}");

                parse_quote! { #identifier : #name }
            }
        })
        .collect();

    let name = format_ident!("{}", to_camel_case(&format!("{}State", state.node_name)));
    let structure = parse_quote!(
        pub struct #name { #(#fields),* }
    );

    let init = init_rust_ast_from_lir(state.init, crates);
    let step = step_rust_ast_from_lir(state.step, crates);
    let implementation = parse_quote!(
        impl #name {
            #init
            #step
        }
    );

    (structure, implementation)
}
