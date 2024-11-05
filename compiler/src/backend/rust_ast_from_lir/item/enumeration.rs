prelude! { just
    macro2::Span,
    lir::item::enumeration::Enumeration,
    conf, syn, parse_quote, Ident,
}

/// Transform LIR enumeration into RustAST enumeration.
pub fn rust_ast_from_lir(enumeration: Enumeration) -> syn::ItemEnum {
    let attribute: syn::Attribute = if conf::greusot() {
        // todo: when v0.1.1 then parse_quote!(#[derive(prelude::Clone, Copy, prelude::PartialEq, prelude::Default, DeepModel)])
        parse_quote!(#[derive(prelude::Clone, Copy, prelude::PartialEq, prelude::Default, DeepModel)])
    } else {
        parse_quote!(#[derive(Clone, Copy, PartialEq, Default)])
    };
    syn::ItemEnum {
        attrs: vec![attribute],
        vis: syn::Visibility::Public(Default::default()),
        enum_token: Default::default(),
        ident: Ident::new(&enumeration.name, Span::call_site()),
        generics: Default::default(),
        brace_token: Default::default(),
        variants: enumeration
            .elements
            .iter()
            .enumerate()
            .map(|(index, element)| {
                let attrs: Vec<syn::Attribute> = if index == 0 {
                    vec![parse_quote!(#[default])]
                } else {
                    vec![]
                };
                syn::Variant {
                    attrs,
                    ident: Ident::new(element, Span::call_site()),
                    fields: syn::Fields::Unit,
                    discriminant: Default::default(),
                }
            })
            .collect(),
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    prelude! { just
        backend::rust_ast_from_lir::item::enumeration::rust_ast_from_lir,
        lir::item::enumeration::Enumeration,
        parse_quote,
    }

    #[test]
    fn should_create_rust_ast_enumeration_from_lir_enumeration() {
        let enumeration =
            Enumeration::new("Color", vec!["Blue".into(), "Red".into(), "Green".into()]);

        let control = parse_quote! {
        #[derive(Clone, Copy, PartialEq, Default)]
        pub enum Color {
            #[default]
            Blue,
            Red,
            Green
        }};
        assert_eq!(rust_ast_from_lir(enumeration), control)
    }
}
