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

pub struct EnumerationTokens<'a> {
    e: &'a Enumeration,
    public: bool,
    greusot: bool,
}
impl Enumeration {
    pub fn prepare_tokens(&self, public: bool, greusot: bool) -> EnumerationTokens<'_> {
        EnumerationTokens {
            e: self,
            public,
            greusot,
        }
    }
}

impl ToTokens for EnumerationTokens<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if self.greusot {
            quote!(#[derive(prelude::Clone, Copy, prelude::PartialEq, DeepModel)]).to_tokens(tokens)
        } else {
            quote!(#[derive(Clone, Copy, PartialEq, Default, Debug)]).to_tokens(tokens)
        }
        let name = &self.e.name;
        let variants = self.e.elements.iter().enumerate().map(|(index, element)| {
            let attr = if !self.greusot && (index == 0) {
                Some(quote!(# [default]))
            } else {
                None
            };
            quote! { #attr #element }
        });
        let pub_token = if self.public {
            quote! {pub}
        } else {
            quote! {}
        };
        quote! {
            #pub_token enum #name {
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
        .prepare_tokens(true, false)
        .to_token_stream();

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
