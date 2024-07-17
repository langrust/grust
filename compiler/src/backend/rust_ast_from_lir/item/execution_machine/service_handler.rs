prelude! {
    macro2::Span,
    syn::*,
    quote::format_ident,
    backend::rust_ast_from_lir::{
        item::execution_machine::{flows_context, instruction_flow},
        r#type::rust_ast_from_lir as type_rust_ast_from_lir,
    },
    lir::item::execution_machine::{
        service_handler::{ FlowHandler, ServiceHandler },
        ArrivingFlow,
    },
}

/// Transform LIR run-loop into an async function performing a loop over events.
pub fn rust_ast_from_lir(run_loop: ServiceHandler) -> Item {
    let ServiceHandler {
        service,
        components,
        flows_handling,
        flows_context,
    } = run_loop;

    // result
    let mut items = flows_context::rust_ast_from_lir(flows_context);

    // store all inputs in a service_store
    let mut service_store_fields: Vec<Field> = vec![];
    let mut service_store_is_somes: Vec<Expr> = vec![];
    flows_handling
        .iter()
        .for_each(|FlowHandler { arriving_flow, .. }| match arriving_flow {
            ArrivingFlow::Channel(flow_name, flow_type, _) => {
                let ident = Ident::new(flow_name.as_str(), Span::call_site());
                let ty = type_rust_ast_from_lir(flow_type.clone());
                service_store_fields
                    .push(parse_quote! { #ident: Option<(#ty, std::time::Instant)> });
                service_store_is_somes.push(parse_quote! { self.#ident.is_some() });
            }
            ArrivingFlow::Period(time_flow_name) | ArrivingFlow::Deadline(time_flow_name) => {
                let ident = Ident::new(time_flow_name.as_str(), Span::call_site());
                service_store_fields
                    .push(parse_quote! { #ident: Option<((), std::time::Instant)> });
                service_store_is_somes.push(parse_quote! { self.#ident.is_some() });
            }
            ArrivingFlow::ServiceDelay(_) | ArrivingFlow::ServiceTimeout(_) => (),
        });
    // service store
    let service_store_name = format_ident!("{}", to_camel_case(&format!("{service}ServiceStore")));
    items.push(Item::Struct(parse_quote! {
        #[derive(Default)]
        pub struct #service_store_name {
            #(#service_store_fields),*
        }
    }));
    // tells is the service_store is not empty
    items.push(Item::Impl(parse_quote! {
        impl #service_store_name {
            pub fn not_empty(&self) -> bool {
                #(#service_store_is_somes)||*
            }
        }
    }));

    // create service structure
    let mut service_fields: Vec<Field> = vec![
        parse_quote! { context: Context },
        parse_quote! { delayed: bool },
        parse_quote! { input_store: #service_store_name },
    ];
    let mut field_values: Vec<FieldValue> = vec![
        parse_quote! { context },
        parse_quote! { delayed },
        parse_quote! { input_store },
    ];
    // with components states
    components.iter().for_each(|component_name| {
        let component_state_struct =
            format_ident!("{}", to_camel_case(&format!("{}State", component_name)));
        let component_name = format_ident!("{}", component_name);
        service_fields.push(parse_quote! { #component_name: #component_state_struct });
        field_values.push(parse_quote! { #component_name });
    });
    // and sending channels
    service_fields.push(parse_quote! { output: futures::channel::mpsc::Sender<O> });
    service_fields
        .push(parse_quote! { timer: futures::channel::mpsc::Sender<(T, std::time::Instant)> });
    field_values.push(parse_quote! { output });
    field_values.push(parse_quote! { timer });
    let service_name = format_ident!("{}", to_camel_case(&format!("{service}Service")));
    items.push(Item::Struct(parse_quote! {
        pub struct #service_name {
            #(#service_fields),*
        }
    }));

    // implement the service with `init` and handler functions
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

    // `init` function
    impl_items.push(parse_quote! {
        pub fn init(
            output: futures::channel::mpsc::Sender<O>,
            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>
        ) -> #service_name {
            let context = Context::init();
            let delayed = true;
            let input_store = Default::default();
            #(#components_states)*
            #service_name {
                #(#field_values),*
            }
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
                ArrivingFlow::Channel(flow_name, flow_type, _) => {
                    let ident = Ident::new(flow_name.as_str(), Span::call_site());
                    let instant = format_ident!("{flow_name}_instant");
                    let function_name: Ident = format_ident!("handle_{flow_name}");
                    let ty = type_rust_ast_from_lir(flow_type);
                    let instructions = instructions
                        .into_iter()
                        .map(instruction_flow_rust_ast_from_lir);
                    let message = syn::LitStr::new(format!("{flow_name} changes too frequently").as_str(), Span::call_site());
                    impl_items.push(parse_quote! {
                        pub async fn #function_name(&mut self, #instant: std::time::Instant, #ident: #ty) -> Result<(), futures::channel::mpsc::SendError> {
                            if self.delayed {
                                self.reset_time_constrains(#instant).await?;
                                #(#instructions)*
                            } else {
                                let unique = self.input_store.#ident.replace((#ident, #instant));
                                assert!(unique.is_none(), #message);
                            }
                            Ok(())
                        }
                    })
                }
                ArrivingFlow::Period(time_flow_name) | ArrivingFlow::Deadline(time_flow_name) => {
                    let ident = Ident::new(time_flow_name.as_str(), Span::call_site());
                    let instant = format_ident!("{time_flow_name}_instant");
                    let function_name: Ident = format_ident!("handle_{time_flow_name}");
                    let instructions = instructions
                        .into_iter()
                        .map(instruction_flow_rust_ast_from_lir);
                    let message = syn::LitStr::new(format!("{time_flow_name} changes too frequently").as_str(), Span::call_site());
                    impl_items.push(parse_quote! {
                        pub async fn #function_name(&mut self,  #instant: std::time::Instant) -> Result<(), futures::channel::mpsc::SendError> {
                            if self.delayed {
                                self.reset_time_constrains(#instant).await?;
                                #(#instructions)*
                            } else {
                                let unique = self.input_store.#ident.replace(((), #instant));
                                assert!(unique.is_none(), #message);
                            }
                            Ok(())
                        }
                    })
                }
                ArrivingFlow::ServiceDelay(service_delay) => {
                    let function_name: Ident = format_ident!("handle_{service_delay}");
                    let instructions = instructions
                        .into_iter()
                        .map(instruction_flow_rust_ast_from_lir);
                    impl_items.push(parse_quote! {
                        pub async fn #function_name(&mut self, instant: std::time::Instant) -> Result<(), futures::channel::mpsc::SendError> {
                            #(#instructions)*
                            Ok(())
                        }
                    });
                    let enum_ident = Ident::new(
                        to_camel_case(service_delay.as_str()).as_str(),
                        Span::call_site(),
                    );
                    impl_items.push(parse_quote! {
                        #[inline]
                        pub async fn reset_service_delay(&mut self, instant: std::time::Instant) -> Result<(), futures::channel::mpsc::SendError> {
                            self.timer.send((T::#enum_ident, instant)).await?;
                            Ok(())
                        }
                    })
                }
                ArrivingFlow::ServiceTimeout(service_timeout) => {
                    let instant = format_ident!("{service_timeout}_instant");
                    let function_name: Ident = format_ident!("handle_{service_timeout}");
                    let instructions = instructions
                        .into_iter()
                        .map(instruction_flow_rust_ast_from_lir);
                    impl_items.push(parse_quote! {
                        pub async fn #function_name(&mut self, #instant: std::time::Instant) -> Result<(), futures::channel::mpsc::SendError> {
                            self.reset_time_constrains(#instant).await?;
                            #(#instructions)*
                            Ok(())
                        }
                    });
                    let enum_ident = Ident::new(
                        to_camel_case(service_timeout.as_str()).as_str(),
                        Span::call_site(),
                    );
                    impl_items.push(parse_quote! {
                        #[inline]
                        pub async fn reset_service_timeout(&mut self, instant: std::time::Instant) -> Result<(), futures::channel::mpsc::SendError> {
                            self.timer.send((T::#enum_ident, instant)).await?;
                            Ok(())
                        }
                    })
                }
            }
        },
    );

    // service handlers in an implementation block
    items.push(Item::Impl(parse_quote! {
        impl #service_name {
            #(#impl_items)*
            #[inline]
            pub async fn reset_time_constrains(&mut self, instant: std::time::Instant) -> Result<(), futures::channel::mpsc::SendError> {
                self.reset_service_delay(instant).await?;
                self.reset_service_timeout(instant).await?;
                self.delayed = false;
                Ok(())
            }
            #[inline]
            pub async fn send_output(&mut self, output: O) -> Result<(), futures::channel::mpsc::SendError> {
                self.output.send(output).await?;
                Ok(())
            }
            #[inline]
            pub async fn send_timer(&mut self, timer: T, instant: std::time::Instant) -> Result<(), futures::channel::mpsc::SendError> {
                self.timer.send((timer, instant)).await?;
                Ok(())
            }
        }
    }));

    // service module
    let module_name = format_ident!("{service}_service");
    Item::Mod(parse_quote! {
       pub mod #module_name {
            use futures::{stream::StreamExt, sink::SinkExt};
            use super::*;

            #(#items)*
       }
    })
}
