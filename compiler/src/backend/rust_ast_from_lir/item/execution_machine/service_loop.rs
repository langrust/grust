use crate::backend::rust_ast_from_lir::{
    item::execution_machine::instruction_flow::rust_ast_from_lir as instruction_flow_rust_ast_from_lir,
    r#type::rust_ast_from_lir as type_rust_ast_from_lir,
};
use crate::common::convert_case::camel_case;
use crate::lir::item::execution_machine::service_loop::{
    ArrivingFlow, FlowHandler, InterfaceFlow, ServiceLoop, TimingEvent, TimingEventKind,
};
use proc_macro2::{Span, TokenStream};
use quote::format_ident;
use syn::*;

/// Transform LIR run-loop into an async function performing a loop over events.
pub fn rust_ast_from_lir(run_loop: ServiceLoop) -> Vec<Item> {
    let ServiceLoop {
        service,
        components,
        input_flows,
        timing_events,
        output_flows,
        flows_handling,
    } = run_loop;

    // result
    let mut items = vec![];

    // create service structure
    let mut fields: Vec<Field> = vec![parse_quote! { context: Context }];
    let mut field_values: Vec<FieldValue> = vec![parse_quote! { context }];
    components.iter().for_each(|component_name| {
        let component_state_struct =
            format_ident!("{}", camel_case(&format!("{}State", component_name)));
        let component_name = format_ident!("{}", component_name);
        fields.push(parse_quote! { #component_name: #component_state_struct });
        field_values.push(parse_quote! { #component_name });
    });
    timing_events
        .iter()
        .for_each(|TimingEvent { identifier, kind }| {
            let ident = format_ident!("{}", identifier);
            match kind {
                TimingEventKind::Period(_) => {
                    fields.push(parse_quote! { #ident: tokio::time::Interval });
                    field_values.push(parse_quote! { #ident });
                }
                TimingEventKind::Timeout(_) => {
                    fields.push(parse_quote! { #ident: tokio::time::Sleep });
                    field_values.push(parse_quote! { #ident });
                }
            }
        });
    input_flows.clone().into_iter().for_each(
        |InterfaceFlow {
             identifier, r#type, ..
         }| {
            let name = Ident::new((identifier + "_channel").as_str(), Span::call_site());
            let ty = type_rust_ast_from_lir(r#type);
            fields.push(parse_quote! { #name: tokio::sync::mpsc::Receiver<#ty> });
            field_values.push(parse_quote! { #name });
        },
    );
    output_flows.clone().into_iter().for_each(
        |InterfaceFlow {
             identifier, r#type, ..
         }| {
            let name = Ident::new((identifier + "_channel").as_str(), Span::call_site());
            let ty = type_rust_ast_from_lir(r#type);
            fields.push(parse_quote! { #name: tokio::sync::mpsc::Sender<#ty> });
            field_values.push(parse_quote! { #name });
        },
    );
    let service_name = format_ident!("{}", camel_case(&format!("{service}Service")));
    items.push(Item::Struct(parse_quote! {
        pub struct #service_name {
            #(#fields),*
        }
    }));

    // implement the service with `new`, `run_loop` and branchs functions
    let mut impl_items: Vec<ImplItem> = vec![];

    // inputs for the `new` and `run_loop` functions
    let mut inputs: Vec<FnArg> = vec![];
    let mut input_values: Vec<Expr> = vec![];
    input_flows.into_iter().for_each(
        |InterfaceFlow {
             identifier, r#type, ..
         }| {
            let name = Ident::new((identifier + "_channel").as_str(), Span::call_site());
            let ty = type_rust_ast_from_lir(r#type);
            inputs.push(parse_quote! { #name: tokio::sync::mpsc::Receiver<#ty> });
            input_values.push(parse_quote! { #name });
        },
    );
    output_flows.into_iter().for_each(
        |InterfaceFlow {
             identifier, r#type, ..
         }| {
            let name = Ident::new((identifier + "_channel").as_str(), Span::call_site());
            let ty = type_rust_ast_from_lir(r#type);
            inputs.push(parse_quote! { #name: tokio::sync::mpsc::Sender<#ty> });
            input_values.push(parse_quote! { #name });
        },
    );

    // create components states
    let components_states = components.into_iter().map(|component_name| {
        let component_state_struct =
            format_ident!("{}", camel_case(&format!("{}State", component_name)));
        let component_name = format_ident!("{}", component_name);
        let state: Stmt = parse_quote! {
            let #component_name = #component_state_struct::init();
        };
        state
    });
    // create time flows
    let time_flows = timing_events
        .into_iter()
        .map(|TimingEvent { identifier, kind }| {
            let ident = format_ident!("{}", identifier);match kind {
            TimingEventKind::Period(period) => {
                let period = Expr::Lit(ExprLit {
                    attrs: vec![],
                    lit: syn::Lit::Int(LitInt::new(&format!("{period}u64"), Span::call_site())),
                });
                let set_period: Stmt =  parse_quote! {
                    let #ident = tokio::time::interval(std::time::Duration::from_millis(#period));
                };
                set_period
            }
            TimingEventKind::Timeout(deadline) => {
                let deadline = Expr::Lit(ExprLit {
                    attrs: vec![],
                    lit: syn::Lit::Int(LitInt::new(&format!("{deadline}u64"), Span::call_site())),
                });
                let set_timeout: Stmt =  parse_quote! {
                    let #ident = tokio::time::sleep_until(tokio::time::Interval::now() + std::time::Duration::from_millis(#deadline));
                };
                set_timeout
            }
        }});
    // instanciate input context
    let context: Stmt = parse_quote! {
        let context = Context::init();
    };
    // `new` function
    impl_items.push(parse_quote! {
        fn new(#(#inputs),*) -> #service_name {
            #(#components_states)*
            #(#time_flows)*
            #context
            #service_name {
                #(#field_values),*
            }
        }
    });

    // loop on the [tokio::select!] macro
    let loop_select: Stmt = {
        let arms = flows_handling.iter().map(
            |FlowHandler {
                 arriving_flow,
                 ..
             }|
             -> TokenStream {
                match arriving_flow {
                    ArrivingFlow::Channel(flow_name, _) => {
                        let ident: Ident = Ident::new(flow_name.as_str(), Span::call_site());
                        let channel: Ident =format_ident!("{flow_name}_channel");
                        let function_name: Ident = format_ident!("handle_{flow_name}");
                        parse_quote! {
                            #ident = service.#channel.recv() => service.#function_name(#ident.unwrap()).await,
                        }
                    }
                    ArrivingFlow::TimingEvent(time_flow_name) => {
                        let ident: Ident = Ident::new(time_flow_name.as_str(), Span::call_site());
                        let function_name: Ident = format_ident!("handle_{time_flow_name}");
                        parse_quote! {
                            _ = service.#ident.tick() => service.#function_name().await,
                        }
                    }
                }
            },
        );
        parse_quote! {
            loop{
                tokio::select! {
                    #(#arms)*
                }
            }
        }
    };
    // `run_loop` function
    impl_items.push(parse_quote! {
        pub async fn run_loop(#(#inputs),*) {
            let mut service = #service_name::new(#(#input_values),*);
            #loop_select
        }
    });

    // flows handler functions
    flows_handling.into_iter().for_each(
        |FlowHandler {
             arriving_flow,
             instructions,
         }| {
            match arriving_flow {
                ArrivingFlow::Channel(flow_name, flow_type) => {
                    let ident: Ident = Ident::new(flow_name.as_str(), Span::call_site());
                    let function_name: Ident = format_ident!("handle_{flow_name}");
                    let ty = type_rust_ast_from_lir(flow_type);
                    let instructions = instructions
                        .into_iter()
                        .map(instruction_flow_rust_ast_from_lir);
                    impl_items.push(parse_quote! {
                        async fn #function_name(&mut self, #ident: #ty) {
                            #(#instructions)*
                        }
                    })
                }
                ArrivingFlow::TimingEvent(time_flow_name) => {
                    let function_name: Ident = format_ident!("handle_{time_flow_name}");
                    let instructions = instructions
                        .into_iter()
                        .map(instruction_flow_rust_ast_from_lir);
                    impl_items.push(parse_quote! {
                        async fn #function_name(&mut self) {
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

    items
}
