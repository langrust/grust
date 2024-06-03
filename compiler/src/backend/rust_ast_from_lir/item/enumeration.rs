prelude! { just
    macro2::Span,
    syn::*,
    lir::item::enumeration::Enumeration,
}

/// Transform LIR enumeration into RustAST enumeration.
pub fn rust_ast_from_lir(enumeration: Enumeration) -> ItemEnum {
    let attribute = parse_quote!(#[derive(Clone, Copy, Debug, PartialEq, Default)]);
    ItemEnum {
        attrs: vec![attribute],
        vis: Visibility::Public(Default::default()),
        enum_token: Default::default(),
        ident: Ident::new(&enumeration.name, Span::call_site()),
        generics: Default::default(),
        brace_token: Default::default(),
        variants: enumeration
            .elements
            .iter()
            .enumerate()
            .map(|(index, element)| {
                let attrs: Vec<Attribute> = if index == 0 {
                    vec![parse_quote!(#[default])]
                } else {
                    vec![]
                };
                Variant {
                    attrs,
                    ident: Ident::new(element, Span::call_site()),
                    fields: Fields::Unit,
                    discriminant: Default::default(),
                }
            })
            .collect(),
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    prelude! { just
        syn::*,
        backend::rust_ast_from_lir::item::enumeration::rust_ast_from_lir,
        lir::item::enumeration::Enumeration,
    }

    #[test]
    fn should_create_rust_ast_enumeration_from_lir_enumeration() {
        let enumeration =
            Enumeration::new("Color", vec!["Blue".into(), "Red".into(), "Green".into()]);

        let control = parse_quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Default)]
        pub enum Color {
            #[default]
            Blue,
            Red,
            Green
        }};
        assert_eq!(rust_ast_from_lir(enumeration), control)
    }
}
