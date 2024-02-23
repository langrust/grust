use std::collections::BTreeSet;
use crate::backend::rust_ast_from_lir::item::node_file::state::init::rust_ast_from_lir as init_rust_ast_from_lir;
use crate::backend::rust_ast_from_lir::item::node_file::state::step::rust_ast_from_lir as step_rust_ast_from_lir;
use crate::backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir;
use crate::common::convert_case::camel_case;
use crate::lir::item::node_file::state::{State, StateElement};
use quote::format_ident;
use syn::*;
/// RustAST init method construction from LIR init.
pub mod init;
/// RustAST step method construction from LIR step.
pub mod step;

/// Transform LIR state into RustAST structure and implementation.
pub fn rust_ast_from_lir(state: State, crates: &mut BTreeSet<String>) -> (ItemStruct, ItemImpl) {
    let fields: Vec<Field> = state
        .elements
        .into_iter()
        .map(|element| match element {
            StateElement::Buffer { identifier, r#type } => {
                let identifier = format_ident!("{identifier}");
                let ty = type_rust_ast_from_lir(r#type);
                parse_quote! { #identifier : #ty }
            }
            StateElement::CalledNode {
                identifier,
                node_name,
            } => {
                let name = format_ident!("{}", camel_case(&format!("{}State", node_name)));
                let identifier = format_ident!("{identifier}");

                parse_quote! { #identifier : #name }
            }
        })
        .collect();

    let name = format_ident!("{}", camel_case(&format!("{}State", state.node_name)));
    let structure = parse_quote!(
        pub struct #name { #(#fields),* }
    );

    let init = init_rust_ast_from_lir(state.init);
    let step = step_rust_ast_from_lir(state.step, crates);
    let implementation = parse_quote!(
        impl #name {
            #init
            #step
        }
    );

    (structure, implementation)
}
