use crate::backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir;
use crate::lir::item::structure::Structure;
use proc_macro2::Span;
use syn::*;
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
    parse_quote! { #[derive(Clone, Copy, Debug, PartialEq)] pub struct #name { #(#fields),* } }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::backend::rust_ast_from_lir::item::structure::rust_ast_from_lir;
    use crate::common::r#type::Type;
    use crate::lir::item::structure::Structure;
    use syn::*;

    #[test]
    fn should_create_rust_ast_structure_from_lir_structure() {
        let structure = Structure {
            name: String::from("Point"),
            fields: vec![
                (String::from("x"), Type::Integer),
                (String::from("y"), Type::Integer),
            ],
        };

        let control = parse_quote! {
            #[derive(Clone, Copy, Debug, PartialEq)]
            pub struct Point {
                pub x: i64,
                pub y: i64
            }
        };
        assert_eq!(rust_ast_from_lir(structure), control)
    }
}
