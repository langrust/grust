//! [ArrayAlias] module.

prelude! {}

/// An array alias definition.
#[derive(Debug, PartialEq)]
pub struct ArrayAlias {
    /// The array's name.
    pub name: Ident,
    /// The array's type.
    pub array_type: Typ,
    /// The array's size.
    pub size: usize,
}

mk_new! { impl ArrayAlias =>
    new {
        name: impl Into<Ident> = name.into(),
        array_type: Typ,
        size: usize,
    }
}

impl ArrayAlias {
    pub fn into_syn(self) -> syn::ItemType {
        let size = self.size;
        syn::ItemType {
            attrs: Default::default(),
            vis: syn::Visibility::Public(Default::default()),
            type_token: Default::default(),
            ident: self.name,
            generics: Default::default(),
            eq_token: Default::default(),
            ty: Box::new(syn::Type::Array(syn::TypeArray {
                bracket_token: Default::default(),
                elem: Box::new(self.array_type.into_syn()),
                semi_token: Default::default(),
                len: parse_quote! { #size},
            })),
            semi_token: Default::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_rust_ast_type_alias_from_ir2_array_alias() {
        let array_alias = ArrayAlias {
            name: Loc::test_id("Matrix5x5"),
            array_type: Typ::array(Typ::int(), 5),
            size: 5,
        };

        let control = parse_quote! { pub type Matrix5x5 = [[i64; 5usize]; 5usize];};
        assert_eq!(array_alias.into_syn(), control)
    }
}
