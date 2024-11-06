//! LIR [Enumeration] module.

prelude! {}

/// An enumeration definition.
#[derive(Debug, PartialEq)]
pub struct Enumeration {
    /// The enumeration's name.
    pub name: String,
    /// The enumeration's elements.
    pub elements: Vec<String>,
}

mk_new! { impl Enumeration =>
    new {
        name: impl Into<String> = name.into(),
        elements: Vec<String>
    }
}

impl Enumeration {
    /// Transform LIR enumeration into RustAST enumeration.
    pub fn into_syn(self) -> syn::ItemEnum {
        let attribute: syn::Attribute = if conf::greusot() {
            // todo: when v0.1.1 then parse_quote!(#[derive(prelude::Clone, Copy, prelude::PartialEq, prelude::Default, DeepModel)])
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
            ident: Ident::new(&self.name, Span::call_site()),
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
                        ident: Ident::new(element, Span::call_site()),
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
    fn should_create_rust_ast_enumeration_from_lir_enumeration() {
        let enumeration =
            Enumeration::new("Color", vec!["Blue".into(), "Red".into(), "Green".into()]);

        let control = parse_quote! {
        #[derive(Clone, Copy, PartialEq, Default)]
        pub enum Color {
            #[default]
            Blue,
            Red,
            Green
        }};
        assert_eq!(enumeration.into_syn(), control)
    }
}
