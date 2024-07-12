prelude! {
    macro2::Span,
    syn::*,
    quote::format_ident,
    lir::item::execution_machine::{
        ArrivingFlow,
        runtime_loop::{ RuntimeLoop, InputHandler },
    },
}

/// Transform LIR run-loop into an async function performing a loop over events.
pub fn rust_ast_from_lir(run_loop: RuntimeLoop) -> ImplItem {
    let RuntimeLoop { input_handlers } = run_loop;

    // init timers
    let init_timers = input_handlers
        .iter()
        .filter_map(|input_flow| -> Option<Stmt> {
            match &input_flow.arriving_flow {
                ArrivingFlow::Channel(_, _, _) => None,
                ArrivingFlow::Period(time_flow_name) | ArrivingFlow::Deadline(time_flow_name) => {
                    let enum_ident = Ident::new(
                        to_camel_case(time_flow_name.as_str()).as_str(),
                        Span::call_site(),
                    );
                    Some(parse_quote!({
                        let res = runtime.timer.send((T::#enum_ident, init_instant)).await;
                        if res.is_err() {return}
                    }))
                }
            }
        });
    // loop on the input stream
    let async_loop: Stmt = {
        let mut input_arms: Vec<Arm> = vec![];
        input_handlers.iter().for_each(
            |InputHandler{
                arriving_flow,
                services,
            }| {
                match arriving_flow {
                    ArrivingFlow::Channel(flow_name, _, _) => {
                        let enum_ident = Ident::new(to_camel_case(flow_name.as_str()).as_str(), Span::call_site());
                        let ident = Ident::new(flow_name.as_str(), Span::call_site());
                        let function_name: Ident = format_ident!("handle_{flow_name}");
                        let call_services_handlers = services.iter().map(|service_name| -> syn::Stmt {
                            let service_name = Ident::new(service_name, Span::call_site());
                            parse_quote! { runtime.#service_name.#function_name(instant, #ident).await; }
                        });
                        input_arms.push(parse_quote! {
                            I::#enum_ident(#ident, instant) => {
                                #(#call_services_handlers)*
                            }
                        })
                    },
                    ArrivingFlow::Period(time_flow_name) | ArrivingFlow::Deadline(time_flow_name) => {
                        let enum_ident = Ident::new(to_camel_case(time_flow_name.as_str()).as_str(), Span::call_site());
                        let function_name: Ident = format_ident!("handle_{time_flow_name}");
                        let call_services_handlers = services.iter().map(|service_name| -> syn::Stmt {
                            let service_name = Ident::new(service_name, Span::call_site());
                            parse_quote! { runtime.#service_name.#function_name(instant).await; }
                        });
                        input_arms.push(parse_quote! {
                            I::Timer(T::#enum_ident, instant) => {
                                #(#call_services_handlers)*
                            }
                        })
                    }
                }
            },
        );
        // parse the loop
        parse_quote! {
            while let Some(input) = input.next().await {
                match input {
                    #(#input_arms),*
                }
            }
        }
    };

    // `run_loop` function
    ImplItem::Fn(parse_quote! {
        pub async fn run_loop(self, init_instant: std::time::Instant, input: impl futures::Stream<Item = I>) {
            futures::pin_mut!(input);
            let mut runtime = self;
            #(#init_timers)*
            #async_loop
        }
    })
}
