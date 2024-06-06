prelude! {
    macro2::{Span, TokenStream},
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
            match kind {
                TimingEventKind::Period(_) => {
                    service_fields.push(parse_quote! { #ident: tokio::time::Interval });
                    field_values.push(parse_quote! { #ident });
                }
                TimingEventKind::Timeout(_) => (),
            }
        });
    input_flows.into_iter().for_each(
        |InterfaceFlow {
             identifier, r#type, ..
         }| {
            let name = Ident::new(&identifier, Span::call_site());
            let ty = type_rust_ast_from_lir(r#type);
            input_variants.push(parse_quote! { #name(#ty) });
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
    service_fields.push(parse_quote! { output: tokio::sync::mpsc::Sender<O> });
    field_values.push(parse_quote! { output });
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
        fn new(output: tokio::sync::mpsc::Sender<O>) -> #service_name {
            let context = Context::init();
            #(#components_states)*
            #(#periods)*
            #service_name {
                #(#field_values),*
            }
        }
    });

    // create deadline time flows
    let deadlines = timing_events
    .iter()
    .filter_map(|TimingEvent { identifier, kind }| {
        let ident = format_ident!("{}", identifier);match kind {
        TimingEventKind::Period(_) => None,
        TimingEventKind::Timeout(deadline) =>{
            let deadline = Expr::Lit(ExprLit {
                attrs: vec![],
                lit: syn::Lit::Int(LitInt::new(&format!("{deadline}u64"), Span::call_site())),
            });
            let set_timeout: Stmt =  parse_quote! {
                let #ident = tokio::time::sleep_until(tokio::time::Instant::now() + tokio::time::Duration::from_millis(#deadline));
            };
            let pin_timeout = parse_quote! { tokio::pin!(#ident); };
            Some(vec![set_timeout, pin_timeout])
        }
    }}).flatten();
    // loop on the [tokio::select!] macro
    let loop_select: Stmt = {
        let mut timers_arms: Vec<TokenStream> = vec![];
        let mut input_arms: Vec<Arm> = vec![];
        flows_handling.iter().for_each(
            |FlowHandler {
                 arriving_flow,
                 deadline_args,
                 ..
             }| {
                match arriving_flow {
                    ArrivingFlow::Channel(flow_name, _) => {
                        let ident: Ident = Ident::new(flow_name.as_str(), Span::call_site());
                        let function_name: Ident = format_ident!("handle_{flow_name}");
                        let mut fn_args: Vec<Expr> = vec![parse_quote!(#ident)];
                        deadline_args.into_iter().for_each(|deadline_name| {
                            let deadline_ident: Ident = format_ident!("{deadline_name}");
                            fn_args.push(
                                parse_quote!(#deadline_ident.as_mut()),
                            )
                        });
                        input_arms.push(parse_quote! {
                            I::#ident(#ident) => service.#function_name(#(#fn_args),*).await
                        })
                    }
                    ArrivingFlow::Period(time_flow_name) => {
                        let ident: Ident = Ident::new(time_flow_name.as_str(), Span::call_site());
                        let function_name: Ident = format_ident!("handle_{time_flow_name}");
                        let fn_args = deadline_args.into_iter().map(|deadline_name| -> Expr {
                            let deadline_ident: Ident = format_ident!("{deadline_name}");
                            parse_quote!(#deadline_ident.as_mut())
                        });
                        timers_arms.push(parse_quote! {
                            _ = service.#ident.tick() => service.#function_name(#(#fn_args),*).await,
                        })
                    }
                    ArrivingFlow::Deadline(time_flow_name) => {
                        let ident: Ident = Ident::new(time_flow_name.as_str(), Span::call_site());
                        let function_name: Ident = format_ident!("handle_{time_flow_name}");
                        let fn_args = deadline_args.into_iter().map(|deadline_name| -> Expr{
                            let deadline_ident: Ident = format_ident!("{deadline_name}");
                            parse_quote!(#deadline_ident.as_mut())
                        });
                        timers_arms.push(parse_quote! {
                            _ = #ident.as_mut() => service.#function_name(#(#fn_args),*).await,
                        })
                    }
                }
            },
        );
        parse_quote! {
            loop{
                tokio::select! {
                    input = input.next() => match input.unwrap() {
                        #(#input_arms),*
                    },
                    #(#timers_arms)*
                }
            }
        }
    };
    // `run_loop` function
    impl_items.push(parse_quote! {
        pub async fn run_loop(self, input: impl futures::Stream<Item = I>) {
            use futures::StreamExt;
            tokio::pin!(input);
            let mut service = self;
            #(#deadlines)*
            #loop_select
        }
    });

    // flows handler functions
    flows_handling.into_iter().for_each(
        |FlowHandler {
             arriving_flow,
             deadline_args,
             instructions,
         }| {
            match arriving_flow {
                ArrivingFlow::Channel(flow_name, flow_type) => {
                    let ident: Ident = Ident::new(flow_name.as_str(), Span::call_site());
                    let function_name: Ident = format_ident!("handle_{flow_name}");
                    let ty = type_rust_ast_from_lir(flow_type);
                    let mut fn_args: Vec<FnArg> =
                        vec![parse_quote!(&mut self), parse_quote!(#ident: #ty)];
                    deadline_args.into_iter().for_each(|deadline_name| {
                        let deadline_ident: Ident = format_ident!("{deadline_name}");
                        fn_args.push(
                            parse_quote!(#deadline_ident: std::pin::Pin<&mut tokio::time::Sleep>),
                        )
                    });
                    let instructions = instructions
                        .into_iter()
                        .map(instruction_flow_rust_ast_from_lir);
                    impl_items.push(parse_quote! {
                        async fn #function_name(#(#fn_args),*) {
                            #(#instructions)*
                        }
                    })
                }
                ArrivingFlow::Period(time_flow_name) | ArrivingFlow::Deadline(time_flow_name) => {
                    let function_name: Ident = format_ident!("handle_{time_flow_name}");
                    let mut fn_args: Vec<FnArg> = vec![parse_quote!(&mut self)];
                    deadline_args.into_iter().for_each(|deadline_name| {
                        let deadline_ident: Ident = format_ident!("{deadline_name}");
                        fn_args.push(
                            parse_quote!(#deadline_ident: std::pin::Pin<&mut tokio::time::Sleep>),
                        )
                    });
                    let instructions = instructions
                        .into_iter()
                        .map(instruction_flow_rust_ast_from_lir);
                    impl_items.push(parse_quote! {
                        async fn #function_name(#(#fn_args),*) {
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
        mod #module_name {
            use super::*;
            use #service_input_name as I;
            use #service_output_name as O;

            #(#items)*
        }
     })
}
