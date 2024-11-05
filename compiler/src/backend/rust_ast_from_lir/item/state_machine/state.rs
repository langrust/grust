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
    lir::item::state_machine::{State, StateElm},
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
            StateElm::Buffer { ident, data: typ } => {
                let ident = format_ident!("{ident}");
                let ty = type_rust_ast_from_lir(typ);
                parse_quote! { #ident : #ty }
            }
            StateElm::CalledNode {
                memory_ident,
                node_name,
            } => {
                let name = format_ident!("{}", to_camel_case(&format!("{}State", node_name)));
                let memory_ident = format_ident!("{memory_ident}");

                parse_quote! { #memory_ident : #name }
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
