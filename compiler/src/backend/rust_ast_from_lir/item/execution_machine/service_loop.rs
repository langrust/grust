use crate::backend::rust_ast_from_lir::{
    item::execution_machine::instruction_flow::rust_ast_from_lir as instruction_flow_rust_ast_from_lir,
    r#type::rust_ast_from_lir as type_rust_ast_from_lir,
};
use crate::common::convert_case::camel_case;
use crate::lir::item::execution_machine::service_loop::{
    ArrivingFlow, FlowHandler, InterfaceFlow, ServiceLoop, TimingEvent, TimingEventKind,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, TokenStreamExt};
use syn::punctuated::Punctuated;
use syn::*;

/// Transform LIR run-loop into items.
pub fn rust_ast_from_lir(run_loop: ServiceLoop) -> Vec<syn::Item> {
    let ServiceLoop {
        service,
        components,
        input_flows,
        timing_events,
        output_flows,
        flows_handling,
    } = run_loop;

    let mut items = vec![];

    // inputs are channels's receivers
    let mut inputs = Punctuated::new();
    input_flows.into_iter().for_each(
        |InterfaceFlow {
             identifier, r#type, ..
         }| {
            let name = Ident::new((identifier + "_channel").as_str(), Span::call_site());
            let ty = type_rust_ast_from_lir(r#type);
            inputs.push(FnArg::Typed(PatType {
                attrs: vec![],
                pat: parse_quote! { mut #name },
                colon_token: Default::default(),
                ty: Box::new(parse_quote! { tokio::sync::mpsc::Receiver<#ty> }),
            }));
        },
    );
    // outputs are channels's senders
    output_flows.into_iter().for_each(
        |InterfaceFlow {
             identifier, r#type, ..
         }| {
            let name = Ident::new((identifier + "_channel").as_str(), Span::call_site());
            let ty = type_rust_ast_from_lir(r#type);
            inputs.push(FnArg::Typed(PatType {
                attrs: vec![],
                pat: parse_quote! { mut #name },
                colon_token: Default::default(),
                ty: Box::new(parse_quote! { tokio::sync::mpsc::Sender<#ty> }),
            }));
        },
    );

    // the async function is called 'run_{service}_loop'
    let sig = syn::Signature {
        constness: None,
        asyncness: Some(Default::default()),
        unsafety: None,
        abi: None,
        fn_token: Default::default(),
        ident: Ident::new(&format!("run_{service}_loop"), Span::call_site()),
        generics: Default::default(),
        paren_token: Default::default(),
        inputs,
        variadic: None,
        output: ReturnType::Default,
    };

    // initiate body statement
    let mut body_stmts = vec![];

    // create components states
    components.into_iter().for_each(|component_name| {
        let component_state_struct =
            format_ident!("{}", camel_case(&format!("{}State", component_name)));
        let component_name = format_ident!("{}", component_name);
        let state = parse_quote! {
            let #component_name = #component_state_struct::init();
        };
        body_stmts.push(state);
    });

    // create time flows
    timing_events
        .into_iter()
        .for_each(|TimingEvent { identifier, kind }| {
            let ident = format_ident!("{}", identifier);match kind {
            TimingEventKind::Period(period) => {
                let period = Expr::Lit(ExprLit {
                    attrs: vec![],
                    lit: syn::Lit::Int(LitInt::new(&format!("{period}u64"), Span::call_site())),
                });
                let set_period =  parse_quote! {
                    let mut #ident = tokio::time::interval(std::time::Duration::from_millis(#period));
                };
                body_stmts.push(set_period);
            }
            TimingEventKind::Timeout(deadline) => {
                let deadline = Expr::Lit(ExprLit {
                    attrs: vec![],
                    lit: syn::Lit::Int(LitInt::new(&format!("{deadline}u64"), Span::call_site())),
                });
                let set_timeout =  parse_quote! {
                    let mut #ident = tokio::time::sleep_until(tokio::time::Interval::now() + std::time::Duration::from_millis(#deadline));
                };
                body_stmts.push(set_timeout);
            }
        }});

    // instanciate input context
    let context = parse_quote! {
        let mut context = Context::init();
    };
    body_stmts.push(context);

    // it performs a loop on the [tokio::select!] macro
    let loop_select = {
        let mut tokens = proc_macro2::TokenStream::new();
        tokens.append_all(flows_handling.into_iter().map(
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
                        let mut tokens_instructions = proc_macro2::TokenStream::new();
                        tokens_instructions.append_all(instructions);
                        parse_quote! {
                            #ident = #channel.recv() => {
                                let #ident = #ident.unwrap();
                                #tokens_instructions
                            }
                        }
                    }
                    ArrivingFlow::TimingEvent(time_flow_name) => {
                        let ident: Ident = Ident::new(time_flow_name.as_str(), Span::call_site());
                        let instructions = instructions
                            .into_iter()
                            .map(instruction_flow_rust_ast_from_lir);
                        let mut tokens_instructions = proc_macro2::TokenStream::new();
                        tokens_instructions.append_all(instructions);
                        parse_quote! {
                            _ = #ident.tick() => {
                                #tokens_instructions
                            }
                        }
                    }
                }
            },
        ));
        let select = Stmt::Expr(
            Expr::Macro(ExprMacro {
                attrs: Default::default(),
                mac: Macro {
                    path: parse_quote! { tokio::select },
                    bang_token: Default::default(),
                    delimiter: MacroDelimiter::Brace(Default::default()),
                    tokens,
                },
            }),
            None,
        );
        Stmt::Expr(
            Expr::Loop(ExprLoop {
                attrs: Default::default(),
                label: Default::default(),
                loop_token: Default::default(),
                body: syn::Block {
                    stmts: vec![select],
                    brace_token: Default::default(),
                },
            }),
            None,
        )
    };
    body_stmts.push(loop_select);

    let item_run_loop = Item::Fn(ItemFn {
        attrs: Default::default(),
        vis: Visibility::Public(Default::default()),
        sig,
        block: Box::new(syn::Block {
            stmts: body_stmts,
            brace_token: Default::default(),
        }),
    });
    items.push(item_run_loop);

    items
}
