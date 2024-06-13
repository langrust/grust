prelude! { just
    macro2::Span,
    syn::*,
    backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir,
    lir::item::structure::Structure,
    conf
}

/// Transform LIR structure into RustAST structure.
pub fn rust_ast_from_lir(structure: Structure) -> ItemStruct {
    let fields = structure.fields.into_iter().map(|(name, r#type)| {
        let name = Ident::new(&name, Span::call_site());
        let r#type = type_rust_ast_from_lir(r#type);
        Field {
            attrs: vec![],
            vis: Visibility::Public(Default::default()),
            ident: Some(name),
            colon_token: Default::default(),
            ty: r#type,
            mutability: FieldMutability::None,
        }
    });
    let name = Ident::new(&structure.name, Span::call_site());
    let attribute: Attribute = if conf::greusot() {
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
        syn::*,
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
