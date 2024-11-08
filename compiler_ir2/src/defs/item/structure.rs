//! [Structure] module.

prelude! {}

/// A structure definition.
#[derive(Debug, PartialEq)]
pub struct Structure {
    /// The structure's name.
    pub name: String,
    /// The structure's fields.
    pub fields: Vec<(String, Typ)>,
}

mk_new! { impl Structure =>
    new {
        name: impl Into<String> = name.into(),
        fields: Vec<(String, Typ)>,
    }
}

impl Structure {
    /// Transform [ir2] structure into RustAST structure.
    pub fn into_syn(self) -> syn::ItemStruct {
        let fields = self.fields.into_iter().map(|(name, typ)| {
            let name = Ident::new(&name, Span::call_site());
            let typ = typ.into_syn();
            syn::Field {
                attrs: vec![],
                vis: syn::Visibility::Public(Default::default()),
                ident: Some(name),
                colon_token: Default::default(),
                ty: typ,
                mutability: syn::FieldMutability::None,
            }
        });
        let name = Ident::new(&self.name, Span::call_site());
        let attribute: syn::Attribute = if conf::greusot() {
            parse_quote!(
                #[derive(prelude::Clone, Copy, prelude::PartialEq, prelude::Default, DeepModel)]
            )
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_rust_ast_structure_from_ir2_structure() {
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
        assert_eq!(structure.into_syn(), control)
    }
}
