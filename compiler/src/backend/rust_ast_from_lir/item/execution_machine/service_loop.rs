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
use syn::punctuated::Punctuated;
use syn::*;

/// Transform LIR run-loop into an async function performing a loop over events.
pub fn rust_ast_from_lir(run_loop: ServiceLoop) -> syn::Item {
    let ServiceLoop {
        service,
        components,
        input_flows,
        timing_events,
        output_flows,
        flows_handling,
    } = run_loop;

    // inputs are channels's receivers
    let mut inputs: Punctuated<FnArg, token::Comma> = Punctuated::new();
    input_flows.into_iter().for_each(
        |InterfaceFlow {
             identifier, r#type, ..
         }| {
            let name = Ident::new((identifier + "_channel").as_str(), Span::call_site());
            let ty = type_rust_ast_from_lir(r#type);
            inputs.push(parse_quote! { mut #name: tokio::sync::mpsc::Receiver<#ty> });
        },
    );
    // outputs are channels's senders
    output_flows.into_iter().for_each(
        |InterfaceFlow {
             identifier, r#type, ..
         }| {
            let name = Ident::new((identifier + "_channel").as_str(), Span::call_site());
            let ty = type_rust_ast_from_lir(r#type);
            inputs.push(parse_quote! { #name: tokio::sync::mpsc::Sender<#ty> });
        },
    );

    // the async function is called 'run_{service}_loop'
    let service_loop_name = Ident::new(&format!("run_{service}_loop"), Span::call_site());

    // create components states
    let components_states = components.into_iter().map(|component_name| {
        let component_state_struct =
            format_ident!("{}", camel_case(&format!("{}State", component_name)));
        let component_name = format_ident!("{}", component_name);
        let state: Stmt = parse_quote! {
            let mut #component_name = #component_state_struct::init();
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
                    let mut #ident = tokio::time::interval(std::time::Duration::from_millis(#period));
                };
                set_period
            }
            TimingEventKind::Timeout(deadline) => {
                let deadline = Expr::Lit(ExprLit {
                    attrs: vec![],
                    lit: syn::Lit::Int(LitInt::new(&format!("{deadline}u64"), Span::call_site())),
                });
                let set_timeout: Stmt =  parse_quote! {
                    let mut #ident = tokio::time::sleep_until(tokio::time::Interval::now() + std::time::Duration::from_millis(#deadline));
                };
                set_timeout
            }
        }});

    // instanciate input context
    let context: Stmt = parse_quote! {
        let mut context = Context::init();
    };

    // it performs a loop on the [tokio::select!] macro
    let loop_select: Stmt = {
        let arms = flows_handling.into_iter().map(
            |FlowHandler {
                 arriving_flow,
                 instructions,
             }|
             -> TokenStream {
                match arriving_flow {
                    ArrivingFlow::Channel(flow_name) => {
                        let ident: Ident = Ident::new(flow_name.as_str(), Span::call_site());
                        let channel: Ident =
                            Ident::new((flow_name + "_channel").as_str(), Span::call_site());
                        let instructions = instructions
                            .into_iter()
                            .map(instruction_flow_rust_ast_from_lir);
                        parse_quote! {
                            #ident = #channel.recv() => {
                                let #ident = #ident.unwrap();
                                #(#instructions)*
                            }
                        }
                    }
                    ArrivingFlow::TimingEvent(time_flow_name) => {
                        let ident: Ident = Ident::new(time_flow_name.as_str(), Span::call_site());
                        let instructions = instructions
                            .into_iter()
                            .map(instruction_flow_rust_ast_from_lir);
                        parse_quote! {
                            _ = #ident.tick() => {
                                #(#instructions)*
                            }
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

    // create the async function
    let item_run_loop: Item = parse_quote! {
        pub async fn #service_loop_name(#inputs) {
            #(#components_states)*
            #(#time_flows)*
            #context
            #loop_select
        }
    };

    item_run_loop
}
