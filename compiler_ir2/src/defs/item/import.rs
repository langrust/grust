prelude! {}

/// A state-machine import structure.
#[derive(Debug, PartialEq)]
pub struct Import {
    /// The node's name.
    pub name: Ident,
    /// The path of the import.
    pub path: syn::Path,
}

mk_new! { impl Import => new {
    name : impl Into<Ident> = name.into(),
    path : syn::Path,
}}

impl Import {
    pub fn into_syn(self) -> syn::ItemUse {
        let Import { name, path } = self;
        let state_name = name.to_state_ty();
        let input_name = name.to_input_ty();
        parse_quote! { use #path::{ #input_name, #state_name }; }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_rust_ast_use_from_ir2_import() {
        let import = Import {
            name: Loc::test_id("rising_edge"),
            path: parse_quote! { grust::grust_std::rising_edge },
        };

        let control = parse_quote! {
            use grust::grust_std::rising_edge::{ RisingEdgeInput, RisingEdgeState };
        };
        assert_eq!(import.into_syn(), control)
    }
}
