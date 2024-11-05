use std::collections::BTreeSet;

prelude! { just
    lir::item::state_machine::StateMachine,
    syn,
}

use self::input::rust_ast_from_lir as input_rust_ast_from_lir;
use self::state::rust_ast_from_lir as state_rust_ast_from_lir;

/// Rust AST input structure construction from LIR input.
mod input {
    prelude! {
        backend::rust_ast_from_lir::typ::rust_ast_from_lir as type_rust_ast_from_lir,
        lir::item::state_machine::{Input, InputElm},
        quote::format_ident,
    }

    /// Transform LIR input into RustAST structure.
    pub fn rust_ast_from_lir(input: Input) -> syn::ItemStruct {
        let mut fields: Vec<syn::Field> = Vec::new();
        for InputElm { identifier, typ } in input.elements {
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
            lir::item::state_machine::{Input, InputElm},
        }

        #[test]
        fn should_create_rust_ast_structure_from_lir_node_input() {
            let input = Input {
                node_name: format!("Node"),
                elements: vec![InputElm {
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
}
/// RustAST state structure and implementation construction from LIR state.
mod state;

/// Transform LIR state_machine into items.
pub fn rust_ast_from_lir(
    state_machine: StateMachine,
    crates: &mut BTreeSet<String>,
) -> Vec<syn::Item> {
    let mut items = vec![];

    let input_structure = input_rust_ast_from_lir(state_machine.input);
    items.push(syn::Item::Struct(input_structure));

    let (state_structure, state_implementation) =
        state_rust_ast_from_lir(state_machine.state, crates);
    items.push(syn::Item::Struct(state_structure));
    items.push(syn::Item::Impl(state_implementation));

    items
}
