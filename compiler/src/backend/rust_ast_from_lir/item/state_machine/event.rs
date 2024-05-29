use crate::backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir;
use crate::common::{convert_case::camel_case, r#type::Type as GRRustType};
use crate::lir::item::state_machine::event::{Event, EventElement};
use quote::format_ident;
use syn::*;

/// Transform LIR event into RustAST structure.
pub fn rust_ast_from_lir(event: Event) -> ItemEnum {
    let mut elements: Vec<Field> = Vec::new();

    event
        .elements
        .into_iter()
        .for_each(|event_element| match event_element {
            EventElement::InputEvent { identifier, r#type } => {
                let ty = type_rust_ast_from_lir(r#type);
                let identifier = format_ident!("{identifier}");
                elements.push(parse_quote! { #identifier(#ty) });
            }
            EventElement::NoEvent => {
                let identifier = format_ident!("NoEvent");
                elements.push(parse_quote! { #identifier });
            }
        });

    let mut generics: Vec<GenericParam> = vec![];
    for (generic_name, generic_type) in event.generics {
        if let GRRustType::Abstract(arguments, output) = generic_type {
            let arguments = arguments.into_iter().map(type_rust_ast_from_lir);
            let output = type_rust_ast_from_lir(*output);
            let identifier = format_ident!("{generic_name}");
            generics.push(parse_quote! { #identifier: Fn(#(#arguments),*) -> #output });
        } else {
            unreachable!()
        }
    }

    let name = format_ident!("{}", camel_case(&format!("{}Event", event.node_name)));
    if generics.is_empty() {
        parse_quote! {
            pub enum #name {
                #(#elements),*
            }
        }
    } else {
        parse_quote! {
            pub enum #name<#(#generics),*> {
                #(#elements),*
            }
        }
    }
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::backend::rust_ast_from_lir::item::state_machine::event::rust_ast_from_lir;
    use crate::common::r#type::Type;
    use crate::lir::item::state_machine::event::{Event, EventElement};
    use syn::*;

    #[test]
    fn should_create_rust_ast_structure_from_lir_node_event() {
        let event = Event {
            node_name: format!("Node"),
            elements: vec![
                EventElement::InputEvent {
                    identifier: format!("E1"),
                    r#type: Type::Integer,
                },
                EventElement::InputEvent {
                    identifier: format!("E2"),
                    r#type: Type::Float,
                },
                EventElement::NoEvent,
            ],
            generics: vec![],
        };
        let control = parse_quote!(
            pub enum NodeEvent {
                E1(i64),
                E2(f64),
                NoEvent,
            }
        );

        assert_eq!(rust_ast_from_lir(event), control)
    }
}
