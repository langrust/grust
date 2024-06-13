prelude! {
    macro2::Span,
    syn::*,
    quote::format_ident,
    backend::rust_ast_from_lir::{
        item::execution_machine::instruction_flow::rust_ast_from_lir as instruction_flow_rust_ast_from_lir,
        r#type::rust_ast_from_lir as type_rust_ast_from_lir,
    },
    lir::item::execution_machine::service_loop::{
        ArrivingFlow, FlowHandler, InterfaceFlow, ServiceLoop, TimingEvent, TimingEventKind,
    },
}

/// Transform LIR run-loop into an async function performing a loop over events.
pub fn rust_ast_from_lir(run_loop: ServiceLoop) -> Item {
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
    let mut timer_variants: Vec<Variant> = vec![];
    let mut timer_arms: Vec<Arm> = vec![];
    let mut input_variants: Vec<Variant> = vec![];
    let mut output_variants: Vec<Variant> = vec![];
    let mut service_fields: Vec<Field> = vec![parse_quote! { context: Context }];
    let mut field_values: Vec<FieldValue> = vec![parse_quote! { context }];
    components.iter().for_each(|component_name| {
        let component_state_struct =
            format_ident!("{}", to_camel_case(&format!("{}State", component_name)));
        let component_name = format_ident!("{}", component_name);
        service_fields.push(parse_quote! { #component_name: #component_state_struct });
        field_values.push(parse_quote! { #component_name });
    });
    timing_events
        .iter()
        .for_each(|TimingEvent { identifier, kind }| {
            let ident = format_ident!("{}", identifier);
            timer_variants.push(parse_quote! { #ident });
            match kind {
                TimingEventKind::Period(duration) | TimingEventKind::Timeout(duration) => {
                    timer_arms.push(parse_quote! { T::#ident =>  #duration as i64});
                }
            }
        });
    input_flows.into_iter().for_each(
        |InterfaceFlow {
             identifier, r#type, ..
         }| {
            let name = Ident::new(&identifier, Span::call_site());
            let ty = type_rust_ast_from_lir(r#type);
            input_variants.push(parse_quote! { #name(#ty, i64) });
        },
    );
    output_flows.into_iter().for_each(
        |InterfaceFlow {
             identifier, r#type, ..
         }| {
            let name = Ident::new(&identifier, Span::call_site());
            let ty = type_rust_ast_from_lir(r#type);
            output_variants.push(parse_quote! { #name(#ty) });
        },
    );
    if !timer_variants.is_empty() {
        input_variants.push(parse_quote! { timer(T, i64) });
    }
    let service_timer_name = format_ident!("{}", to_camel_case(&format!("{service}ServiceTimer")));
    items.push(Item::Enum(parse_quote! {
        pub enum #service_timer_name {
            #(#timer_variants),*
        }
    }));
    items.push(Item::Impl(parse_quote! {
        impl #service_timer_name {
            pub fn get_timer(&self) -> i64 {
                match self {
                    #(#timer_arms),*
                }
            }
        }
    }));
    let service_input_name = format_ident!("{}", to_camel_case(&format!("{service}ServiceInput")));
    items.push(Item::Enum(parse_quote! {
        pub enum #service_input_name {
            #(#input_variants),*
        }
    }));
    let service_output_name =
        format_ident!("{}", to_camel_case(&format!("{service}ServiceOutput")));
    items.push(Item::Enum(parse_quote! {
        pub enum #service_output_name {
            #(#output_variants),*
        }
    }));
    service_fields.push(parse_quote! { output: futures::channel::mpsc::Sender<O> });
    service_fields.push(parse_quote! { timer: futures::channel::mpsc::Sender<(T, i64)> });
    field_values.push(parse_quote! { output });
    field_values.push(parse_quote! { timer });
    let service_name = format_ident!("{}", to_camel_case(&format!("{service}Service")));
    items.push(Item::Struct(parse_quote! {
        pub struct #service_name {
            #(#service_fields),*
        }
    }));

    // implement the service with `new`, `run_loop` and branchs functions
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
    // create periods time flows
    let periods = timing_events
        .iter()
        .filter_map(|TimingEvent { identifier, kind }| {
            let ident = format_ident!("{}", identifier);match kind {
            TimingEventKind::Period(period) => {
                let period = Expr::Lit(ExprLit {
                    attrs: vec![],
                    lit: syn::Lit::Int(LitInt::new(&format!("{period}u64"), Span::call_site())),
                });
                let set_period: Stmt =  parse_quote! {
                    let #ident = tokio::time::interval(tokio::time::Duration::from_millis(#period));
                };
                Some(set_period)
            }
            TimingEventKind::Timeout(_) => None
        }});
    // `new` function
    impl_items.push(parse_quote! {
        pub fn new(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, i64)>
        ) -> #service_name {
            let context = Context::init();
            #(#components_states)*
            #(#periods)*
            #service_name {
                #(#field_values),*
            }
        }
    });

    // loop on the [tokio::select!] macro
    let loop_select: Stmt = {
        let mut input_arms: Vec<Arm> = vec![];
        flows_handling
            .iter()
            .for_each(|FlowHandler { arriving_flow, .. }| match arriving_flow {
                ArrivingFlow::Channel(flow_name, _) => {
                    let ident: Ident = Ident::new(flow_name.as_str(), Span::call_site());
                    let function_name: Ident = format_ident!("handle_{flow_name}");
                    input_arms.push(parse_quote! {
                        I::#ident(#ident, timestamp) => service.#function_name(timestamp, #ident).await
                    })
                }
                ArrivingFlow::Period(time_flow_name) | ArrivingFlow::Deadline(time_flow_name) => {
                    let ident: Ident = Ident::new(time_flow_name.as_str(), Span::call_site());
                    let function_name: Ident = format_ident!("handle_{time_flow_name}");
                    input_arms.push(parse_quote! {
                        I::timer(T::#ident, timestamp) => service.#function_name(timestamp).await,
                    })
                }
            });
        parse_quote! {
            loop{
                tokio::select! {
                    input = input.next() => if let Some(input) = input {
                        match input {
                            #(#input_arms),*
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    };
    // `run_loop` function
    impl_items.push(parse_quote! {
        pub async fn run_loop(self, input: impl futures::Stream<Item = I>) {
            tokio::pin!(input);
            let mut service = self;
            #loop_select
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
                ArrivingFlow::Channel(flow_name, flow_type) => {
                    let ident: Ident = Ident::new(flow_name.as_str(), Span::call_site());
                    let function_name: Ident = format_ident!("handle_{flow_name}");
                    let ty = type_rust_ast_from_lir(flow_type);
                    let instructions = instructions
                        .into_iter()
                        .map(instruction_flow_rust_ast_from_lir);
                    impl_items.push(parse_quote! {
                        async fn #function_name(&mut self, timestamp: i64, #ident: #ty) {
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
                        async fn #function_name(&mut self, timestamp: i64) {
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
           use #service_timer_name as T;
           use #service_input_name as I;
           use #service_output_name as O;

           #(#items)*
       }
    })
}
