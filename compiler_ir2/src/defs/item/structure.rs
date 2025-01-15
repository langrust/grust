//! [Structure] module.

prelude! {}

/// A structure definition.
#[derive(Debug, PartialEq)]
pub struct Structure {
    /// The structure's name.
    pub name: Ident,
    /// The structure's fields.
    pub fields: Vec<(Ident, Typ)>,
}

mk_new! { impl Structure =>
    new {
        name: impl Into<Ident> = name.into(),
        fields: Vec<(Ident, Typ)>,
    }
}

impl Structure {
    /// Transform [ir2] structure into RustAST structure.
    pub fn into_syn(self, ctx: &ir0::Ctx) -> syn::ItemStruct {
        let fields = self.fields.into_iter().map(|(name, typ)| {
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
        let name = self.name;
        let attribute: syn::Attribute = if ctx.conf.greusot {
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
            Loc::test_id("Point"),
            vec![
                (Loc::test_id("x"), Typ::int()),
                (Loc::test_id("y"), Typ::int()),
            ],
        );

        let control = parse_quote! {
            #[derive(Clone, Copy, PartialEq, Default)]
            pub struct Point {
                pub x: i64,
                pub y: i64
            }
        };
        assert_eq!(structure.into_syn(&ir0::Ctx::empty()), control)
    }
}
