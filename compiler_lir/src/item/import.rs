prelude! {}

/// A state-machine import structure.
#[derive(Debug, PartialEq)]
pub struct Import {
    /// The node's name.
    pub name: String,
    /// The path of the import.
    pub path: syn::Path,
}

mk_new! { impl Import => new {
    name : impl Into<String> = name.into(),
    path : syn::Path,
}}

impl Import {
    pub fn into_syn(self) -> syn::ItemUse {
        let Import { name, path } = self;
        let state_name = format_ident!("{}", to_camel_case(&format!("{}State", name)));
        let input_name = format_ident!("{}", to_camel_case(&format!("{}Input", name)));
        parse_quote! { use #path::{ #input_name, #state_name }; }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_rust_ast_use_from_lir_import() {
        let import = Import {
            name: "rising_edge".to_string(),
            path: parse_quote! { grust::grust_std::rising_edge },
        };

        let control = parse_quote! {
            use grust::grust_std::rising_edge::{ RisingEdgeInput, RisingEdgeState };
        };
        assert_eq!(import.into_syn(), control)
    }
}
