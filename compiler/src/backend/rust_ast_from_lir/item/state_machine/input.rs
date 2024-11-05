prelude! {
    backend::rust_ast_from_lir::typ::rust_ast_from_lir as type_rust_ast_from_lir,
    lir::item::state_machine::input::{Input, InputElement},
    quote::format_ident,
}

/// Transform LIR input into RustAST structure.
pub fn rust_ast_from_lir(input: Input) -> syn::ItemStruct {
    let mut fields: Vec<syn::Field> = Vec::new();
    for InputElement { identifier, typ } in input.elements {
        let typ = type_rust_ast_from_lir(typ);
        let identifier = format_ident!("{identifier}");
        fields.push(parse_quote! { pub #identifier : #typ });
    }

    let name = format_ident!("{}", to_camel_case(&format!("{}Input", input.node_name)));
    parse_quote! {
        pub struct #name {
            #(#fields,)*
        }
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    prelude! {
        backend::rust_ast_from_lir::item::state_machine::input::rust_ast_from_lir,
        lir::item::state_machine::input::{Input, InputElement},
    }

    #[test]
    fn should_create_rust_ast_structure_from_lir_node_input() {
        let input = Input {
            node_name: format!("Node"),
            elements: vec![InputElement {
                identifier: format!("i"),
                typ: Typ::int(),
            }],
        };
        let control = parse_quote!(
            pub struct NodeInput {
                pub i: i64,
            }
        );

        assert_eq!(rust_ast_from_lir(input), control)
    }
}
