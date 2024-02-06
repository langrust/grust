use crate::lir::item::node_file::import::Import;
use proc_macro2::Span;
use syn::*;
/// Transform LIR import into RustAST import.
pub fn rust_ast_from_lir(import: Import) -> ItemUse {
    match import {
        Import::NodeFile(name) => {
            let name = Ident::new(&name, Span::call_site());
            parse_quote! { use crate::#name::*; }
        },
        Import::Function(name) => {
            let name = Ident::new(&name, Span::call_site());
            parse_quote! { use crate::functions::#name; }
        },
        Import::Enumeration(name) => {
            let name = Ident::new(&name, Span::call_site());
            parse_quote! { use crate::typedefs::#name; }
        },
        Import::Structure(name) => {
            let name = Ident::new(&name, Span::call_site());
            parse_quote! { use crate::typedefs::#name; }
        },
        Import::ArrayAlias(name) => {
            let name = Ident::new(&name, Span::call_site());
            parse_quote! { use crate::typedefs::#name; }
        },
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::backend::rust_ast_from_lir::item::node_file::import::rust_ast_from_lir;
    use crate::lir::item::node_file::import::Import;
    use syn::*;

    #[test]
    fn should_create_rust_ast_import_from_lir_function_import() {
        let import = Import::Function(String::from("foo"));

        let control = parse_quote! { use crate::functions::foo; };
        assert_eq!(rust_ast_from_lir(import), control)
    }

    #[test]
    fn should_create_rust_ast_import_from_lir_node_import() {
        let import = Import::NodeFile(String::from("my_node"));

        let control = parse_quote! { use crate::my_node::*; };
        assert_eq!(rust_ast_from_lir(import), control)
    }
}
