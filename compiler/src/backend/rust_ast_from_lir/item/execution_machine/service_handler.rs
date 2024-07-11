prelude! {
    macro2::Span,
    syn::*,
    quote::format_ident,
    backend::rust_ast_from_lir::{
        item::execution_machine::{
            flows_context::rust_ast_from_lir as flows_context_rust_ast_from_lir,
            instruction_flow::rust_ast_from_lir as instruction_flow_rust_ast_from_lir
        },
        r#type::rust_ast_from_lir as type_rust_ast_from_lir,
    },
    lir::item::execution_machine::{
        service_handler::{ FlowHandler, ServiceHandler },
        ArrivingFlow,
    },
}

/// Transform LIR run-loop into an async function performing a loop over events.
pub fn rust_ast_from_lir(run_loop: ServiceHandler) -> Item {
    let ServiceHandler {
        service,
        components,
        flows_handling,
        flows_context,
    } = run_loop;

    // result
    let mut items = flows_context_rust_ast_from_lir(flows_context);

    // create service structure
    let mut service_fields: Vec<Field> = vec![parse_quote! { context: Context }];
    let mut field_values: Vec<FieldValue> = vec![parse_quote! { context }];
    // with components states
    components.iter().for_each(|component_name| {
        let component_state_struct =
            format_ident!("{}", to_camel_case(&format!("{}State", component_name)));
        let component_name = format_ident!("{}", component_name);
        service_fields.push(parse_quote! { #component_name: #component_state_struct });
        field_values.push(parse_quote! { #component_name });
    });
    // and sending channels
    service_fields.push(parse_quote! { output: futures::channel::mpsc::Sender<O> });
    service_fields
        .push(parse_quote! { timer: futures::channel::mpsc::Sender<(T, std::time::Instant)> });
    field_values.push(parse_quote! { output });
    field_values.push(parse_quote! { timer });
    let service_name = format_ident!("{}", to_camel_case(&format!("{service}Service")));
    items.push(Item::Struct(parse_quote! {
        pub struct #service_name {
            #(#service_fields),*
        }
    }));

    // implement the service with `init` and handler functions
    let mut impl_items: Vec<ImplItem> = vec![];

    // create components states
    let components_states = components.into_iter().map(|component_name| {
        let component_state_struct =
            format_ident!("{}", to_camel_case(&format!("{}State", component_name)));
        let component_name = format_ident!("{}", component_name);
        let state: Stmt = parse_quote! {
            let #component_name = #component_state_struct::init();
        };
        state
    });
    // `init` function
    impl_items.push(parse_quote! {
        pub fn init(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>
        ) -> #service_name {
            let context = Context::init();
            #(#components_states)*
            #service_name {
                #(#field_values),*
            }
        }
    });

    // flows handler functions
    flows_handling.into_iter().for_each(
        |FlowHandler {
             arriving_flow,
             instructions,
             ..
         }| {
            match arriving_flow {
                ArrivingFlow::Channel(flow_name, flow_type, _) => {
                    let ident: Ident = Ident::new(flow_name.as_str(), Span::call_site());
                    let function_name: Ident = format_ident!("handle_{flow_name}");
                    let ty = type_rust_ast_from_lir(flow_type);
                    let instructions = instructions
                        .into_iter()
                        .map(instruction_flow_rust_ast_from_lir);
                    impl_items.push(parse_quote! {
                        pub async fn #function_name(&mut self, instant: std::time::Instant, #ident: #ty) {
                            #(#instructions)*
                        }
                    })
                }
                ArrivingFlow::Period(time_flow_name) | ArrivingFlow::Deadline(time_flow_name) => {
                    let function_name: Ident = format_ident!("handle_{time_flow_name}");
                    let instructions = instructions
                        .into_iter()
                        .map(instruction_flow_rust_ast_from_lir);
                    impl_items.push(parse_quote! {
                        pub async fn #function_name(&mut self, instant: std::time::Instant) {
                            #(#instructions)*
                        }
                    })
                }
            }
        },
    );

    items.push(parse_quote! {
        impl #service_name {
            #(#impl_items)*
        }
    });

    let module_name = format_ident!("{service}_service");
    Item::Mod(parse_quote! {
       pub mod #module_name {
            use futures::{stream::StreamExt, sink::SinkExt};
            use super::*;

            #(#items)*
       }
    })
}
