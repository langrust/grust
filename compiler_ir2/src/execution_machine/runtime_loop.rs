prelude! { execution_machine::ArrivingFlow }

/// The runtime loop structure.
#[derive(Debug, PartialEq, Default)]
pub struct RuntimeLoop {
    /// The input flow handlers.
    pub input_handlers: Vec<InputHandler>,
}

impl RuntimeLoop {
    /// Transform [ir2] run-loop into an async function performing a loop over events.
    pub fn into_syn(self) -> syn::ImplItem {
        // init timers
        let init_timers = self.input_handlers
        .iter()
        .filter_map(|input_flow| -> Option<syn::Stmt> {
            match &input_flow.arriving_flow {
                ArrivingFlow::Channel(_, _, _) | ArrivingFlow::ServiceDelay(_) => None,
                ArrivingFlow::Period(time_flow_name)
                | ArrivingFlow::Deadline(time_flow_name)
                | ArrivingFlow::ServiceTimeout(time_flow_name) => {
                    let enum_ident = Ident::new(
                        &to_camel_case(time_flow_name.to_string()),
                        Span::call_site(),
                    );
                    Some(parse_quote! { runtime.send_timer(T::#enum_ident, init_instant).await?; })
                }
            }
        });
        // loop on the input stream
        let async_loop: syn::Stmt = {
            let mut input_arms: Vec<syn::Arm> = vec![];
            for InputHandler {
                arriving_flow,
                services,
            } in self.input_handlers.iter()
            {
                match arriving_flow {
                    ArrivingFlow::Channel(flow_name, _, _) => {
                        let enum_ident =
                            Ident::new(&to_camel_case(flow_name.to_string()), Span::call_site());
                        let ident = flow_name;
                        let function_name: Ident = format_ident!("handle_{flow_name}");
                        let call_services_handlers =
                            services.iter().map(|service_name| -> syn::Stmt {
                                parse_quote! {
                                    runtime.#service_name.#function_name(instant, #ident).await?;
                                }
                            });
                        input_arms.push(parse_quote! {
                            I::#enum_ident(#ident, instant) => {
                                #(#call_services_handlers)*
                            }
                        })
                    }
                    ArrivingFlow::Period(time_flow_name)
                    | ArrivingFlow::Deadline(time_flow_name)
                    | ArrivingFlow::ServiceDelay(time_flow_name)
                    | ArrivingFlow::ServiceTimeout(time_flow_name) => {
                        let enum_ident = Ident::new(
                            to_camel_case(time_flow_name.to_string()).as_str(),
                            Span::call_site(),
                        );
                        let function_name: Ident = format_ident!("handle_{time_flow_name}");
                        let call_services_handlers =
                            services.iter().map(|service_name| -> syn::Stmt {
                                parse_quote! {
                                    runtime.#service_name.#function_name(instant).await?;
                                }
                            });
                        input_arms.push(parse_quote! {
                            I::Timer(T::#enum_ident, instant) => {
                                #(#call_services_handlers)*
                            }
                        })
                    }
                }
            }
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
        syn::ImplItem::Fn(parse_quote! {
            pub async fn run_loop(
                self, init_instant: std::time::Instant, input: impl futures::Stream<Item = I>
            ) -> Result<(), futures::channel::mpsc::SendError> {
                futures::pin_mut!(input);
                let mut runtime = self;
                #(#init_timers)*
                #async_loop
                Ok(())
            }
        })
    }
}

/// A flow structure.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InputHandler {
    /// Arriving flow.
    pub arriving_flow: ArrivingFlow,
    /// Delivered services.
    pub services: Vec<Ident>,
}
