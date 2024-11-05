prelude! {
    macro2::Span,
    quote::format_ident,
    backend::rust_ast_from_lir::{
        typ::rust_ast_from_lir as type_rust_ast_from_lir,
    },
    lir::item::execution_machine::{
        InterfaceFlow, ExecutionMachine, TimingEvent, TimingEventKind
    },
}

use runtime_loop::rust_ast_from_lir as runtime_loop_rust_ast_from_lir;
use service_handler::rust_ast_from_lir as service_handler_rust_ast_from_lir;

pub mod flow_expression;
pub mod flows_context;
pub mod instruction_flow;
pub mod runtime_loop;
pub mod service_handler;

/// Transform LIR execution-machine into a runtime module.
pub fn rust_ast_from_lir(execution_machine: ExecutionMachine) -> syn::Item {
    let ExecutionMachine {
        input_flows,
        output_flows,
        timing_events,
        runtime_loop,
        services_handlers,
    } = execution_machine;

    let (runtime_items, field_values) = {
        let mut runtime_items = vec![];

        // create runtime structures and their implementations
        let mut timer_variants: Vec<syn::Variant> = vec![];
        let mut timer_duration_arms: Vec<syn::Arm> = vec![];
        let mut timer_reset_arms: Vec<syn::Arm> = vec![];
        let mut input_variants: Vec<syn::Variant> = vec![];
        let mut input_eq_arms: Vec<syn::Arm> = vec![];
        let mut input_get_instant_arms: Vec<syn::Arm> = vec![];
        let mut output_variants: Vec<syn::Variant> = vec![];
        let mut runtime_fields: Vec<syn::Field> = vec![];
        let mut field_values: Vec<syn::FieldValue> = vec![];

        for TimingEvent { identifier, kind } in timing_events.iter() {
            let enum_ident = Ident::new(
                to_camel_case(identifier.as_str()).as_str(),
                Span::call_site(),
            );
            timer_variants.push(parse_quote! { #enum_ident });
            match kind {
                TimingEventKind::Period(duration) => {
                    timer_duration_arms.push(parse_quote! { T::#enum_ident => {
                        std::time::Duration::from_millis(#duration)
                    } });
                    timer_reset_arms.push(parse_quote! { T::#enum_ident => false });
                }
                TimingEventKind::Timeout(duration)
                | TimingEventKind::ServiceTimeout(duration)
                | TimingEventKind::ServiceDelay(duration) => {
                    timer_duration_arms.push(parse_quote! { T::#enum_ident => {
                        std::time::Duration::from_millis(#duration)
                    } });
                    timer_reset_arms.push(parse_quote! { T::#enum_ident => true });
                }
            }
        }

        for InterfaceFlow {
            identifier, typ, ..
        } in input_flows.iter()
        {
            let enum_ident = Ident::new(
                to_camel_case(identifier.as_str()).as_str(),
                Span::call_site(),
            );
            let ty = type_rust_ast_from_lir(typ.clone());
            input_variants.push(parse_quote! { #enum_ident(#ty, std::time::Instant) });
            input_eq_arms.push(
                    parse_quote! { (I::#enum_ident(this, _), I::#enum_ident(other, _)) => this.eq(other) },
                );
            input_get_instant_arms.push(parse_quote! { I::#enum_ident(_, instant) => *instant });
        }

        for InterfaceFlow {
            identifier, typ, ..
        } in output_flows.into_iter()
        {
            let enum_ident = Ident::new(
                to_camel_case(identifier.as_str()).as_str(),
                Span::call_site(),
            );
            let ty = type_rust_ast_from_lir(typ);
            output_variants.push(parse_quote! { #enum_ident(#ty, std::time::Instant) });
        }

        if !timer_variants.is_empty() {
            input_variants.push(parse_quote! { Timer(T, std::time::Instant) });
            input_eq_arms
                .push(parse_quote! { (I::Timer(this, _), I::Timer(other, _)) => this.eq(other) });
            input_get_instant_arms.push(parse_quote! { I::Timer(_, instant) => *instant });
        }
        input_eq_arms.push(parse_quote! { _ => false });
        runtime_items.push(syn::Item::Enum(parse_quote! {
            #[derive(PartialEq)]
            pub enum RuntimeTimer {
                #(#timer_variants),*
            }
        }));
        runtime_items.push(syn::Item::Impl(parse_quote! {
            impl timer_stream::Timing for RuntimeTimer {
                fn get_duration(&self) -> std::time::Duration {
                    match self {
                        #(#timer_duration_arms),*
                    }
                }
                fn do_reset(&self) -> bool {
                    match self {
                        #(#timer_reset_arms),*
                    }
                }
            }
        }));
        runtime_items.push(syn::Item::Enum(parse_quote! {
            pub enum RuntimeInput {
                #(#input_variants),*
            }
        }));
        if !timer_variants.is_empty() {
            runtime_items.push(parse_quote! {
                impl priority_stream::Reset for RuntimeInput {
                    fn do_reset(&self) -> bool {
                        match self {
                                I::Timer(timer, _) => timer_stream::Timing::do_reset(timer),
                                _ => false,
                        }
                    }
                }
            });
        }
        runtime_items.push(parse_quote! {
            impl PartialEq for RuntimeInput {
                fn eq(&self, other: &Self) -> bool {
                    match (self, other) {
                        #(#input_eq_arms),*
                    }
                }
            }
        });
        runtime_items.push(parse_quote! {
            impl RuntimeInput {
                pub fn get_instant(&self) -> std::time::Instant {
                    match self {
                        #(#input_get_instant_arms),*
                    }
                }
                pub fn order(v1: &Self, v2: &Self) -> std::cmp::Ordering {
                    v1.get_instant().cmp(&v2.get_instant())
                }
            }
        });
        runtime_items.push(syn::Item::Enum(parse_quote! {
            pub enum RuntimeOutput {
                #(#output_variants),*
            }
        }));
        services_handlers.iter().for_each(|service_handler| {
            let service_name = &service_handler.service;
            let service_path = format_ident!("{}_service", service_name);
            let service_state_struct =
                format_ident!("{}", to_camel_case(&format!("{}Service", service_name)));
            let service_name = format_ident!("{}", service_name);
            runtime_fields
                .push(parse_quote! { #service_name: #service_path::#service_state_struct });
            field_values.push(parse_quote! { #service_name });
        });
        runtime_fields
            .push(parse_quote! { timer: futures::channel::mpsc::Sender<(T, std::time::Instant)> });
        field_values.push(parse_quote! { timer });

        runtime_items.push(syn::Item::Struct(parse_quote! {
            pub struct Runtime {
                #(#runtime_fields),*
            }
        }));

        (runtime_items, field_values)
    };

    // function that creates a new runtime
    let new_runtime = {
        let nb_services = services_handlers.len();
        let is_last = |idx| idx < nb_services - 1;
        // initializes services
        let services_init = services_handlers.iter().enumerate().map(|(idx, service_handler)| {
            let service_name = &service_handler.service;
            let service_path = format_ident!("{}_service", service_name);
            let service_state_struct =
                format_ident!("{}", to_camel_case(&format!("{}Service", service_name)));
            let service_name = format_ident!("{}", service_name);
            let output_channel: syn::Expr = if is_last(idx) {
                parse_quote! { output.clone() }
            } else {
                parse_quote! { output }
            };
            let state: syn::Stmt = parse_quote! {
                let #service_name = #service_path::#service_state_struct::init(#output_channel, timer.clone());
            };
            state
        });
        // parse the funtion that creates a new runtime
        syn::ImplItem::Fn(parse_quote! {
            pub fn new(
                output: futures::channel::mpsc::Sender<O>,
                timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>
            ) -> Runtime {
                #(#services_init)*
                Runtime {
                    #(#field_values),*
                }
            }
        })
    };

    // create the runtime loop
    let run_loop = runtime_loop_rust_ast_from_lir(runtime_loop);

    // create the services handlers
    let handlers = services_handlers
        .into_iter()
        .map(service_handler_rust_ast_from_lir);

    // parse the runtime module
    syn::Item::Mod(parse_quote! {
        pub mod runtime {
            use futures::{stream::StreamExt, sink::SinkExt};
            use super::*;
            use RuntimeTimer as T;
            use RuntimeInput as I;
            use RuntimeOutput as O;

            #(#runtime_items)*

            impl Runtime {
                #new_runtime

                #[inline]
                pub async fn send_timer(&mut self, timer: T, instant: std::time::Instant) -> Result<(), futures::channel::mpsc::SendError> {
                    self.timer.send((timer, instant)).await?;
                    Ok(())
                }

                #run_loop
            }

            #(#handlers)*
        }
    })
}
