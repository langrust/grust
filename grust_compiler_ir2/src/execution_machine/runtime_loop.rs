prelude! { execution_machine::{ArrivingFlow, InterfaceFlow} }

/// The runtime loop structure.
#[derive(Debug, PartialEq, Default)]
pub struct RuntimeLoop {
    /// The initialization handlers.
    pub init_handlers: Vec<ServiceInit>,
    /// The input flow handlers.
    pub input_handlers: Vec<ServiceTrigger>,
}

pub struct RuntimeLoopTokens<'a> {
    rl: &'a RuntimeLoop,
    in_flows: &'a [InterfaceFlow],
}
impl RuntimeLoop {
    pub fn prepare_tokens<'a>(&'a self, in_flows: &'a [InterfaceFlow]) -> RuntimeLoopTokens<'a> {
        RuntimeLoopTokens { rl: self, in_flows }
    }
}

impl ToTokens for RuntimeLoopTokens<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // initial signals' values
        let init_args = self
            .in_flows
            .iter()
            .filter(|flow| !flow.typ.is_event())
            .map(|InterfaceFlow { ident, .. }| {
                quote! { #ident }
            });

        // TODO: call init functions of services with initial signals' values
        let run_inits = self.rl.init_handlers.iter().map(
            |ServiceInit {
                 service,
                 input_flows,
             }| {
                let args = input_flows.iter().map(|InterfaceFlow { ident, .. }| ident);
                quote! { runtime.#service.handle_init(#(#args),*).await?; }
            },
        );

        // loop on the input stream
        let async_loop = {
            let input_arms = self.rl.input_handlers.iter().map(
                |ServiceTrigger {
                     arriving_flow,
                     services,
                 }| match arriving_flow {
                    ArrivingFlow::Channel(flow_name, _, _) => {
                        let enum_ident = flow_name.to_ty();
                        let ident = flow_name;
                        let function_name = flow_name.to_handle_fn();
                        let instant = Ident::instant_var();
                        let call_services_handlers = services.iter().map(|service| {
                            quote! {
                                runtime.#service.#function_name(#instant, #ident).await?;
                            }
                        });
                        quote! {
                            I::#enum_ident(#ident, #instant) => { #(#call_services_handlers)* }
                        }
                    }
                    ArrivingFlow::Period(time_flow_name)
                    | ArrivingFlow::Deadline(time_flow_name)
                    | ArrivingFlow::ServiceDelay(time_flow_name)
                    | ArrivingFlow::ServiceTimeout(time_flow_name) => {
                        let enum_ident = time_flow_name.to_camel();
                        let instant = Ident::instant_var();
                        let function_name = time_flow_name.to_handle_fn();
                        let call_services_handlers = services.iter().map(|service_name| {
                            quote! {
                                runtime.#service_name.#function_name(#instant).await?;
                            }
                        });
                        quote! {
                            I::Timer(T::#enum_ident, #instant) => { #(#call_services_handlers)* }
                        }
                    }
                },
            );
            // parse the loop
            quote! {
                while let Some(input) = input.next().await {
                    match input {
                        #(#input_arms),*
                    }
                }
            }
        };

        // `run_loop` function
        quote! {
            pub async fn run_loop(
                self,
                input: impl grust::futures::Stream<Item = I>,
                init_vals: RuntimeInit,
            ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                grust::futures::pin_mut!(input);
                let mut runtime = self;
                let RuntimeInit {
                    #(#init_args),*
                } = init_vals;
                #(#run_inits)*
                #async_loop
                Ok(())
            }
        }
        .to_tokens(tokens)
    }
}

/// Triggers services with arriving flow.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ServiceTrigger {
    /// Arriving flow.
    pub arriving_flow: ArrivingFlow,
    /// Triggered services.
    pub services: Vec<Ident>,
}

/// Initialize the call of the service.
#[derive(Clone, Debug, PartialEq)]
pub struct ServiceInit {
    /// Initialized service.
    pub service: Ident,
    /// Input flows.
    pub input_flows: Vec<InterfaceFlow>,
}

mod old {
    use super::*;

    impl RuntimeLoop {
        /// Transform [ir2] run-loop into an async function performing a loop over events.
        pub fn into_syn(self, output_flows: Vec<InterfaceFlow>) -> syn::ImplItem {
            // init timers
            let init_timers = self.input_handlers
            .iter()
            .filter_map(|input_flow| -> Option<syn::Stmt> {
                match &input_flow.arriving_flow {
                    ArrivingFlow::Channel(_, _, _) | ArrivingFlow::ServiceDelay(_) => None,
                    ArrivingFlow::Period(time_flow_name)
                    | ArrivingFlow::Deadline(time_flow_name)
                    | ArrivingFlow::ServiceTimeout(time_flow_name) => {
                        let enum_ident = time_flow_name.to_camel();
                        let init_instant = Ident::init_instant_var();
                        Some(parse_quote! { runtime.send_timer(T::#enum_ident, runtime.#init_instant).await?; })
                    }
                }
            });
            // init outputs
            let init_outputs = output_flows.into_iter().filter_map(
                |InterfaceFlow { ident, typ, .. }| -> Option<syn::Stmt> {
                    let enum_ident = ident.to_camel();
                    if typ.is_event() {
                        None
                    } else {
                        let init_instant = Ident::init_instant_var();
                        Some(parse_quote! { runtime.send_output(O::#enum_ident(Default::default(), runtime.#init_instant)).await?; })
                    }
                }
            );
            // loop on the input stream
            let async_loop: syn::Stmt = {
                let mut input_arms: Vec<syn::Arm> = vec![];
                for ServiceTrigger {
                    arriving_flow,
                    services,
                } in self.input_handlers.iter()
                {
                    match arriving_flow {
                        ArrivingFlow::Channel(flow_name, _, _) => {
                            let enum_ident = flow_name.to_ty();
                            let ident = flow_name;
                            let function_name = flow_name.to_handle_fn();
                            let instant = Ident::instant_var();
                            let call_services_handlers =
                                services.iter().map(|service_name| -> syn::Stmt {
                                    parse_quote! {
                                        runtime.#service_name.#function_name(#instant, #ident).await?;
                                    }
                                });
                            input_arms.push(parse_quote! {
                                I::#enum_ident(#ident, #instant) => {
                                    #(#call_services_handlers)*
                                }
                            })
                        }
                        ArrivingFlow::Period(time_flow_name)
                        | ArrivingFlow::Deadline(time_flow_name)
                        | ArrivingFlow::ServiceDelay(time_flow_name)
                        | ArrivingFlow::ServiceTimeout(time_flow_name) => {
                            let enum_ident = time_flow_name.to_camel();
                            let instant = Ident::instant_var();
                            let function_name = time_flow_name.to_handle_fn();
                            let call_services_handlers =
                                services.iter().map(|service_name| -> syn::Stmt {
                                    parse_quote! {
                                        runtime.#service_name.#function_name(#instant).await?;
                                    }
                                });
                            input_arms.push(parse_quote! {
                                I::Timer(T::#enum_ident, #instant) => {
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
                    self, input: impl grust::futures::Stream<Item = I>
                ) -> Result<(), grust::futures::channel::mpsc::SendError> {
                    grust::futures::pin_mut!(input);
                    let mut runtime = self;
                    #(#init_timers)*
                    #(#init_outputs)*
                    #async_loop
                    Ok(())
                }
            })
        }
    }
}
