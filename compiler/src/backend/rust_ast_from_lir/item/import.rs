use crate::lir::item::import::Import;
use proc_macro2::Span;
use std::collections::BTreeSet;
use syn::*;
/// Transform LIR import into RustAST import.
pub fn rust_ast_from_lir(import: Import, crates: &mut BTreeSet<String>) -> ItemUse {
    match import {
        Import::StateMachine(name) => {
            let name = Ident::new(&name, Span::call_site());
            parse_quote! { use crate::#name::*; }
        }
        Import::Function(name) => {
            let name = Ident::new(&name, Span::call_site());
            parse_quote! { use crate::functions::#name; }
        }
        Import::Enumeration(name) => {
            let name = Ident::new(&name, Span::call_site());
            parse_quote! { use crate::typedefs::#name; }
        }
        Import::Structure(name) => {
            let name = Ident::new(&name, Span::call_site());
            parse_quote! { use crate::typedefs::#name; }
        }
        Import::ArrayAlias(name) => {
            let name = Ident::new(&name, Span::call_site());
            parse_quote! { use crate::typedefs::#name; }
        }
        Import::Creusot(name) => {
            crates.insert(String::from(
                "creusot-contracts = { path = \"creusot/creusot-contracts\" }",
            ));
            let name = Ident::new(&name, Span::call_site());
            parse_quote! { use creusot_contracts::#name; }
        }
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::backend::rust_ast_from_lir::item::import::rust_ast_from_lir;
    use crate::lir::item::import::Import;
    use syn::*;

    #[test]
    fn should_create_rust_ast_import_from_lir_function_import() {
        let import = Import::Function(String::from("foo"));

        let control = parse_quote! { use crate::functions::foo; };
        assert_eq!(rust_ast_from_lir(import, &mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_import_from_lir_node_import() {
        let import = Import::StateMachine(String::from("my_node"));

        let control = parse_quote! { use crate::my_node::*; };
        assert_eq!(rust_ast_from_lir(import, &mut Default::default()), control)
    }
}
