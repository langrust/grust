prelude! { just
    macro2::Span,
    backend::rust_ast_from_lir::typ::rust_ast_from_lir as type_rust_ast_from_lir,
    lir::item::structure::Structure,
    conf, syn, parse_quote, Ident
}

/// Transform LIR structure into RustAST structure.
pub fn rust_ast_from_lir(structure: Structure) -> syn::ItemStruct {
    let fields = structure.fields.into_iter().map(|(name, typ)| {
        let name = Ident::new(&name, Span::call_site());
        let typ = type_rust_ast_from_lir(typ);
        syn::Field {
            attrs: vec![],
            vis: syn::Visibility::Public(Default::default()),
            ident: Some(name),
            colon_token: Default::default(),
            ty: typ,
            mutability: syn::FieldMutability::None,
        }
    });
    let name = Ident::new(&structure.name, Span::call_site());
    let attribute: syn::Attribute = if conf::greusot() {
        parse_quote!(#[derive(prelude::Clone, Copy, prelude::PartialEq, prelude::Default, DeepModel)])
    } else {
        parse_quote!(#[derive(Clone, Copy, PartialEq, Default)])
    };
    parse_quote! {
        #attribute
        pub struct #name {
            #(#fields),*
        }
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    prelude! {
        backend::rust_ast_from_lir::item::structure::rust_ast_from_lir,
        lir::item::structure::Structure,
    }

    #[test]
    fn should_create_rust_ast_structure_from_lir_structure() {
        let structure = Structure::new(
            "Point",
            vec![("x".into(), Typ::int()), ("y".into(), Typ::int())],
        );

        let control = parse_quote! {
            #[derive(Clone, Copy, PartialEq, Default)]
            pub struct Point {
                pub x: i64,
                pub y: i64
            }
        };
        assert_eq!(rust_ast_from_lir(structure), control)
    }
}
