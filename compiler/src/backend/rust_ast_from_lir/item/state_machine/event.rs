use crate::backend::rust_ast_from_lir::r#type::rust_ast_from_lir as type_rust_ast_from_lir;
use crate::common::{convert_case::camel_case, r#type::Type as GRRustType};
use crate::lir::item::state_machine::event::{Event, EventElement, IntoOtherEvent};
use quote::format_ident;
use syn::*;

/// Transform LIR event into RustAST structure.
pub fn rust_ast_from_lir(event: Event) -> Vec<Item> {
    // create the enumeration elements
    let mut elements: Vec<Variant> = Vec::new();
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

    // maybe generics
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

    // create enumeration
    let event_name = format_ident!("{}", camel_case(&format!("{}Event", event.node_name)));
    let enum_item: ItemEnum = if generics.is_empty() {
        parse_quote! {
            pub enum #event_name {
                #(#elements),*
            }
        }
    } else {
        parse_quote! {
            pub enum #event_name<#(#generics),*> {
                #(#elements),*
            }
        }
    };

    // create the event convertions
    let mut items = event
        .intos
        .into_iter()
        .map(
            |IntoOtherEvent {
                 other_node_name,
                 convertions,
             }| {
                let other_event_name =
                    format_ident!("{}", camel_case(&format!("{}Event", other_node_name)));

                // convert every event element
                let mut arms: Vec<Arm> = convertions
                    .into_iter()
                    .map(|(from, into)| {
                        let from_name = format_ident!("{from}");
                        let into_name = format_ident!("{into}");
                        parse_quote! {
                            #event_name::#from_name(v) => #other_event_name::#into_name(v)
                        }
                    })
                    .collect();
                arms.push(parse_quote! { _ => #other_event_name::NoEvent });

                // create match expression
                let match_expression: ExprMatch = parse_quote! {
                    match self {
                        #(#arms),*
                    }
                };

                // create implementation of 'Into' trait
                let into_impl: ItemImpl = parse_quote! {
                    impl Into<#other_event_name> for #event_name {
                        fn into(self) -> #other_event_name {
                            #match_expression
                        }
                    }
                };

                Item::Impl(into_impl)
            },
        )
        .collect::<Vec<_>>();

    items.insert(0, Item::Enum(enum_item));

    items
}

#[cfg(test)]
mod rust_ast_from_lir {
    use crate::backend::rust_ast_from_lir::item::state_machine::event::rust_ast_from_lir;
    use crate::common::r#type::Type;
    use crate::lir::item::state_machine::event::{Event, EventElement, IntoOtherEvent};
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
            intos: vec![IntoOtherEvent {
                other_node_name: format!("OtherNode"),
                convertions: vec![(format!("E1"), format!("E"))],
            }],
            generics: vec![],
        };
        let enum_control = parse_quote!(
            pub enum NodeEvent {
                E1(i64),
                E2(f64),
                NoEvent,
            }
        );
        let impl_control = parse_quote!(
            impl Into<OtherNodeEvent> for NodeEvent {
                fn into(self) -> OtherNodeEvent {
                    match self {
                        NodeEvent::E1(v) => OtherNodeEvent::E(v),
                        _ => OtherNodeEvent::NoEvent
                    }
                }
            }
        );

        assert_eq!(rust_ast_from_lir(event), vec![enum_control, impl_control])
    }
}
