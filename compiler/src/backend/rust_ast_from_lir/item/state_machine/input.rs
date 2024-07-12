prelude! {
    backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir,
    lir::item::state_machine::input::{Input, InputElement},
    quote::format_ident,
    syn::*,
}

/// Transform LIR input into RustAST structure.
pub fn rust_ast_from_lir(input: Input) -> ItemStruct {
    let mut fields: Vec<Field> = Vec::new();
    for InputElement { identifier, r#type } in input.elements {
        let ty = type_rust_ast_from_lir(r#type);
        let identifier = format_ident!("{identifier}");
        fields.push(parse_quote! { pub #identifier : #ty });
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
    use syn::*;

    #[test]
    fn should_create_rust_ast_structure_from_lir_node_input() {
        let input = Input {
            node_name: format!("Node"),
            elements: vec![InputElement {
                identifier: format!("i"),
                r#type: Typ::int(),
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
