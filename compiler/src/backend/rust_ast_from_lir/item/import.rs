prelude! {
    quote::format_ident,
    lir::item::Import,
}

/// Transform LIR import into RustAST 'use' import.
pub fn rust_ast_from_lir(import: Import) -> syn::ItemUse {
    let Import { name, path } = import;
    let state_name = format_ident!("{}", to_camel_case(&format!("{}State", name)));
    let input_name = format_ident!("{}", to_camel_case(&format!("{}Input", name)));
    parse_quote! { use #path::{ #input_name, #state_name }; }
}

#[cfg(test)]
mod rust_ast_from_lir {
    prelude! {
        backend::rust_ast_from_lir::item::import::rust_ast_from_lir,
        lir::item::Import,
    }

    #[test]
    fn should_create_rust_ast_use_from_lir_import() {
        let import = Import {
            name: "rising_edge".to_string(),
            path: parse_quote! { grust::grust_std::rising_edge },
        };

        let control = parse_quote! { use grust::grust_std::rising_edge::{ RisingEdgeInput, RisingEdgeState }; };
        assert_eq!(rust_ast_from_lir(import), control)
    }
}
