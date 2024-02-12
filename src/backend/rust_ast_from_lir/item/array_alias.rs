use crate::backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir;
use crate::lir::item::array_alias::ArrayAlias;
use proc_macro2::Span;
use syn::*;
/// Transform LIR array alias into RustAST type alias.
pub fn rust_ast_from_lir(array_alias: ArrayAlias) -> ItemType {
    let size = array_alias.size;
    ItemType {
        attrs: Default::default(),
        vis: Visibility::Public(Default::default()),
        type_token: Default::default(),
        ident: Ident::new(&array_alias.name, Span::call_site()),
        generics: Default::default(),
        eq_token: Default::default(),
        ty: Box::new(Type::Array(TypeArray {
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
    use crate::backend::rust_ast_from_lir::item::array_alias::rust_ast_from_lir;
    use crate::common::r#type::Type;
    use crate::lir::item::array_alias::ArrayAlias;

    use syn::*;
    #[test]
    fn should_create_rust_ast_type_alias_from_lir_array_alias() {
        let array_alias = ArrayAlias {
            name: String::from("Matrix5x5"),
            array_type: Type::Array(Box::new(Type::Integer), 5),
            size: 5,
        };

        let control = parse_quote! { pub type Matrix5x5 = [[i64; 5usize]; 5usize];};
        assert_eq!(rust_ast_from_lir(array_alias), control)
    }
}
