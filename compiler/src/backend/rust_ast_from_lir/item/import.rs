use std::collections::BTreeSet;

prelude! { just
    macro2::Span,
    syn::*,
    lir::item::import::Import,
}

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
    prelude! { just
        syn::*,
        backend::rust_ast_from_lir::item::import::rust_ast_from_lir,
        lir::item::import::Import,
    }

    #[test]
    fn should_create_rust_ast_import_from_lir_function_import() {
        let import = Import::function("foo");

        let control = parse_quote! { use crate::functions::foo; };
        assert_eq!(rust_ast_from_lir(import, &mut Default::default()), control)
    }

    #[test]
    fn should_create_rust_ast_import_from_lir_node_import() {
        let import = Import::state_machine("my_node");

        let control = parse_quote! { use crate::my_node::*; };
        assert_eq!(rust_ast_from_lir(import, &mut Default::default()), control)
    }
}
