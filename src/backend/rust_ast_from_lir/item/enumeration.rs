use crate::lir::item::enumeration::Enumeration;
use proc_macro2::Span;
use syn::*;

/// Transform LIR enumeration into RustAST enumeration.
pub fn rust_ast_from_lir(enumeration: Enumeration) -> ItemEnum {
    ItemEnum {
        attrs: Default::default(),
        vis: Visibility::Public(Default::default()),
        enum_token: Default::default(),
        ident: Ident::new(&enumeration.name, Span::call_site()),
        generics: Default::default(),
        brace_token: Default::default(),
        variants: enumeration
            .elements
            .iter()
            .map(|element| Variant {
                attrs: Default::default(),
                ident: Ident::new(element, Span::call_site()),
                fields: Fields::Unit,
                discriminant: Default::default(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::backend::rust_ast_from_lir::item::enumeration::rust_ast_from_lir;
    use crate::lir::item::enumeration::Enumeration;
    use syn::*;

    #[test]
    fn should_create_rust_ast_enumeration_from_lir_enumeration() {
        let enumeration = Enumeration {
            name: String::from("Color"),
            elements: vec![
                String::from("Blue"),
                String::from("Red"),
                String::from("Green"),
            ],
        };

        let control = parse_quote! {
        pub enum Color {
            Blue,
            Red,
            Green
        }};
        assert_eq!(rust_ast_from_lir(enumeration), control)
    }
}
