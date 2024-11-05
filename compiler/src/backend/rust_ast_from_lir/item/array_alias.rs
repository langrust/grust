prelude! { just
    macro2::Span,
    backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir,
    lir::item::array_alias::ArrayAlias,
    syn, parse_quote
}

/// Transform LIR array alias into RustAST type alias.
pub fn rust_ast_from_lir(array_alias: ArrayAlias) -> syn::ItemType {
    let size = array_alias.size;
    syn::ItemType {
        attrs: Default::default(),
        vis: syn::Visibility::Public(Default::default()),
        type_token: Default::default(),
        ident: syn::Ident::new(&array_alias.name, Span::call_site()),
        generics: Default::default(),
        eq_token: Default::default(),
        ty: Box::new(syn::Type::Array(syn::TypeArray {
            bracket_token: Default::default(),
            elem: Box::new(type_rust_ast_from_lir(array_alias.array_type)),
            semi_token: Default::default(),
            len: parse_quote! { #size},
        })),

        semi_token: Default::default(),
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    prelude! {
        backend::rust_ast_from_lir::item::array_alias::rust_ast_from_lir,
        lir::item::array_alias::ArrayAlias,
    }

    #[test]
    fn should_create_rust_ast_type_alias_from_lir_array_alias() {
        let array_alias = ArrayAlias {
            name: "Matrix5x5".into(),
            array_type: Typ::array(Typ::int(), 5),
            size: 5,
        };

        let control = parse_quote! { pub type Matrix5x5 = [[i64; 5usize]; 5usize];};
        assert_eq!(rust_ast_from_lir(array_alias), control)
    }
}
