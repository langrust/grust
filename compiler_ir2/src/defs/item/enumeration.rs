//! [Enumeration] module.

prelude! {}

/// An enumeration definition.
#[derive(Debug, PartialEq)]
pub struct Enumeration {
    /// The enumeration's name.
    pub name: Ident,
    /// The enumeration's elements.
    pub elements: Vec<Ident>,
}

mk_new! { impl Enumeration =>
    new {
        name: impl Into<Ident> = name.into(),
        elements: Vec<Ident>
    }
}

impl Enumeration {
    /// Transform an [ir2] enumeration into a token stream.
    pub fn to_token_stream(&self, ctx: &Ctx) -> TokenStream2 {
        let mut tokens = TokenStream2::new();
        self.to_tokens(ctx, &mut tokens);
        tokens
    }
    /// Writes an [ir2] enumeration to a token stream.
    pub fn to_tokens(&self, ctx: &Ctx, tokens: &mut TokenStream2) {
        if ctx.conf.greusot {
            quote!(#[derive(prelude::Clone, Copy, prelude::PartialEq, DeepModel)]).to_tokens(tokens)
        } else {
            quote!(#[derive(Clone, Copy, PartialEq, Default, Debug)]).to_tokens(tokens)
        }
        let name = &self.name;
        let variants = self.elements.iter().enumerate().map(|(index, element)| {
            let attr = if !ctx.conf.greusot && (index == 0) {
                Some(quote!(# [default]))
            } else {
                None
            };
            quote! { #attr #element }
        });
        quote! {
            pub enum #name {
                #(#variants),*
            }
        }
        .to_tokens(tokens)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_rust_ast_enumeration_from_ir2_enumeration() {
        let enumeration = Enumeration::new(
            Loc::test_id("Color"),
            vec![
                Loc::test_id("Blue"),
                Loc::test_id("Red"),
                Loc::test_id("Green"),
            ],
        )
        .to_token_stream(&Ctx::empty());

        let control = parse_quote! {
        #[derive(Clone, Copy, PartialEq, Default, Debug)]
        pub enum Color {
            #[default]
            Blue,
            Red,
            Green
        }};
        let enumeration: syn::ItemEnum = parse_quote!(#enumeration);
        assert_eq!(enumeration, control)
    }
}
