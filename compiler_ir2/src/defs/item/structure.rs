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
    /// Transform an [ir2] structure into a token stream.
    pub fn to_token_stream(&self, ctx: &Ctx) -> TokenStream2 {
        let mut tokens = TokenStream2::new();
        self.to_tokens(ctx, &mut tokens);
        tokens
    }
    /// Writes an [ir2] structure into a token stream.
    pub fn to_tokens(&self, ctx: &ir0::Ctx, tokens: &mut TokenStream2) {
        if ctx.conf.greusot {
            quote!(
                #[derive(prelude::Clone, Copy, prelude::PartialEq, DeepModel)]
            )
            .to_tokens(tokens)
        } else {
            quote!(#[derive(Clone, Copy, PartialEq, Default, Debug)]).to_tokens(tokens)
        };
        let fields = self
            .fields
            .iter()
            .map(|(name, typ)| quote! { pub #name: #typ });
        let name = &self.name;
        quote! {
            pub struct #name {
                #(#fields),*
            }
        }
        .to_tokens(tokens)
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
        )
        .to_token_stream(&Ctx::empty());

        let control = parse_quote! {
            #[derive(Clone, Copy, PartialEq, Default, Debug)]
            pub struct Point {
                pub x: i64,
                pub y: i64
            }
        };
        let structure: syn::ItemStruct = parse_quote!(#structure);
        assert_eq!(structure, control)
    }
}
