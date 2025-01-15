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
    /// Transform [ir2] enumeration into RustAST enumeration.
    pub fn into_syn(self, ctx: &ir0::Ctx) -> syn::ItemEnum {
        let attribute: syn::Attribute = if ctx.conf.greusot {
            // todo: when v0.1.1 then
            // ```
            // parse_quote!(
            //     #[derive(prelude::Clone, Copy, prelude::PartialEq, prelude::Default, DeepModel)]
            // )
            // ```
            parse_quote!(
                #[derive(prelude::Clone, Copy, prelude::PartialEq, prelude::Default, DeepModel)]
            )
        } else {
            parse_quote!(#[derive(Clone, Copy, PartialEq, Default)])
        };
        syn::ItemEnum {
            attrs: vec![attribute],
            vis: syn::Visibility::Public(Default::default()),
            enum_token: Default::default(),
            ident: self.name,
            generics: Default::default(),
            brace_token: Default::default(),
            variants: self
                .elements
                .iter()
                .enumerate()
                .map(|(index, element)| {
                    let attrs: Vec<syn::Attribute> = if index == 0 {
                        vec![parse_quote!(#[default])]
                    } else {
                        vec![]
                    };
                    syn::Variant {
                        attrs,
                        ident: element.clone(),
                        fields: syn::Fields::Unit,
                        discriminant: Default::default(),
                    }
                })
                .collect(),
        }
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
        );

        let control = parse_quote! {
        #[derive(Clone, Copy, PartialEq, Default)]
        pub enum Color {
            #[default]
            Blue,
            Red,
            Green
        }};
        assert_eq!(enumeration.into_syn(&ir0::Ctx::empty()), control)
    }
}
