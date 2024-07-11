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
                ArrivingFlow::Period(identifier) | ArrivingFlow::Deadline(identifier) => {
                    let timer_ident = format_ident!("{}", identifier);
                    Some(parse_quote!({
                        let res = runtime.timer.send((T::#timer_ident, init_instant)).await;
                        if res.is_err() {return}
                    }))
                }
            }
        });
    // loop on the [tokio::select!] macro
    let loop_select: Stmt = {
        let mut input_arms: Vec<Arm> = vec![];
        input_handlers.iter().for_each(
            |InputHandler{
                arriving_flow,
                services,
            }| {
                match arriving_flow {
                    ArrivingFlow::Channel(flow_name, _, _) => {
                        let ident: Ident = Ident::new(flow_name.as_str(), Span::call_site());
                        let function_name: Ident = format_ident!("handle_{flow_name}");
                        let call_services_handlers = services.iter().map(|service_name| -> syn::Stmt {
                            let service_name: Ident = Ident::new(service_name, Span::call_site());
                            parse_quote! { runtime.#service_name.#function_name(instant, #ident).await; }
                        });
                        input_arms.push(parse_quote! {
                            I::#ident(#ident, instant) => {
                                #(#call_services_handlers)*
                            }
                        })
                    },
                    ArrivingFlow::Period(time_flow_name) | ArrivingFlow::Deadline(time_flow_name) => {
                        let ident: Ident = Ident::new(time_flow_name.as_str(), Span::call_site());
                        let function_name: Ident = format_ident!("handle_{time_flow_name}");
                        let call_services_handlers = services.iter().map(|service_name| -> syn::Stmt {
                            let service_name: Ident = Ident::new(service_name, Span::call_site());
                            parse_quote! { runtime.#service_name.#function_name(instant).await; }
                        });
                        input_arms.push(parse_quote! {
                            I::timer(T::#ident, instant) => {
                                #(#call_services_handlers)*
                            }
                        })
                    }
                }
            },
        );
        // parse the loop
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
    ImplItem::Fn(parse_quote! {
        pub async fn run_loop(self, init_instant: std::time::Instant, input: impl futures::Stream<Item = I>) {
            tokio::pin!(input);
            let mut runtime = self;
            #(#init_timers)*
            #loop_select
        }
    })
}
