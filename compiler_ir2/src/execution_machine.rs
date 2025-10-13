prelude! {}

mod runtime_loop;
mod service_handler;

pub use self::{runtime_loop::*, service_handler::*};

/// A execution-machine structure.
#[derive(Debug, PartialEq, Default)]
pub struct ExecutionMachine {
    /// The input flows.
    pub input_flows: Vec<InterfaceFlow>,
    /// The output flows.
    pub output_flows: Vec<InterfaceFlow>,
    /// The timing events.
    pub timing_events: Vec<TimingEvent>,
    /// The runtime loop.
    pub runtime_loop: RuntimeLoop,
    /// The services handlers.
    pub services_handlers: Vec<ServiceHandler>,
}
pub struct ExecutionMachineTokens<'a> {
    em: &'a ExecutionMachine,
    demo: bool,
    test: bool,
}
impl ExecutionMachine {
    pub fn prepare_tokens(&self, demo: bool, test: bool) -> ExecutionMachineTokens {
        ExecutionMachineTokens {
            em: self,
            demo,
            test,
        }
    }
}
impl ToTokens for ExecutionMachineTokens<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mod_items = {
            let mut tokens = TokenStream2::new();

            // generate tokens corresponding to required imports
            {
                tokens.extend(quote! {
                    use futures::{stream::StreamExt, sink::SinkExt};
                    use super::*;
                });
            }

            // create fields for runtime structures
            let mut timer_variants: Vec<TokenStream2> = vec![];
            let mut timer_duration_arms: Vec<TokenStream2> = vec![];
            let mut timer_reset_arms: Vec<TokenStream2> = vec![];
            let mut input_variants: Vec<TokenStream2> = vec![];
            let mut input_eq_arms: Vec<TokenStream2> = vec![];
            let mut input_get_instant_arms: Vec<TokenStream2> = vec![];
            let mut output_variants: Vec<TokenStream2> = vec![];
            let mut init_fields: Vec<TokenStream2> = vec![];
            let mut runtime_fields: Vec<TokenStream2> = vec![];
            let mut field_values: Vec<TokenStream2> = vec![];
            {
                for TimingEvent { identifier, kind } in self.em.timing_events.iter() {
                    let enum_ident = identifier.to_camel();
                    timer_variants.push(enum_ident.to_token_stream());
                    match kind {
                        TimingEventKind::Period(duration) => {
                            timer_duration_arms.push(quote! { T::#enum_ident => {
                                    std::time::Duration::from_millis(#duration)
                            } });
                            timer_reset_arms.push(quote! { T::#enum_ident => false });
                        }
                        TimingEventKind::Timeout(duration)
                        | TimingEventKind::ServiceTimeout(duration)
                        | TimingEventKind::ServiceDelay(duration) => {
                            timer_duration_arms.push(quote! { T::#enum_ident => {
                                std::time::Duration::from_millis(#duration)
                            } });
                            timer_reset_arms.push(quote! { T::#enum_ident => true });
                        }
                    }
                }

                for InterfaceFlow { ident, typ, .. } in self.em.input_flows.iter() {
                    let enum_ident = ident.to_camel();
                    input_variants.push(quote! { #enum_ident(#typ, std::time::Instant) });
                    input_eq_arms.push(quote! {
                        (I::#enum_ident(this, _), I::#enum_ident(other, _)) => this.eq(other)
                    });
                    let instant = Ident::instant_var();
                    input_get_instant_arms
                        .push(quote! { I::#enum_ident(_, #instant) => *#instant });
                    if !typ.is_event() {
                        init_fields.push(quote! { pub #ident: #typ });
                    }
                }

                for InterfaceFlow { ident, typ, .. } in self.em.output_flows.iter() {
                    let enum_ident = ident.to_camel();
                    output_variants.push(quote! { #enum_ident(#typ, std::time::Instant) });
                }

                if !timer_variants.is_empty() {
                    input_variants.push(parse_quote! { Timer(T, std::time::Instant) });
                    input_eq_arms.push(
                        parse_quote! { (I::Timer(this, _), I::Timer(other, _)) => this.eq(other) },
                    );
                    let instant = Ident::instant_var();
                    input_get_instant_arms
                        .push(parse_quote! { I::Timer(_, #instant) => *#instant });
                }
                input_eq_arms.push(parse_quote! { _ => false });

                for ServiceHandler {
                    service_ident,
                    service_struct_ident,
                    service_mod_ident,
                    ..
                } in self.em.services_handlers.iter()
                {
                    runtime_fields
                        .push(quote! { #service_ident: #service_mod_ident::#service_struct_ident });
                    field_values.push(service_ident.to_token_stream())
                }

                runtime_fields.push(quote!(output : futures::channel::mpsc::Sender<O>));
                field_values.push(quote!(output));
                if !timer_variants.is_empty() {
                    runtime_fields.push(
                        quote! { timer: futures::channel::mpsc::Sender<(T, std::time::Instant)> },
                    );
                    field_values.push(quote!(timer));
                }
            }

            // generate tokens corresponding to runtime structure and their impl
            {
                // runtime input struct
                {
                    quote! {
                        #[derive(Debug)]
                        pub enum RuntimeInput {
                            #(#input_variants),*
                        }
                        use RuntimeInput as I;
                    }
                    .to_tokens(&mut tokens);
                    let timer_reset = if !timer_variants.is_empty() {
                        quote! {I::Timer(timer, _) => timer_stream::Timing::do_reset(timer),}
                    } else {
                        quote! {}
                    };
                    quote! {
                        impl priority_stream::Reset for RuntimeInput {
                            fn do_reset(&self) -> bool {
                                match self {
                                        #timer_reset
                                        _ => false,
                                }
                            }
                        }
                    }
                    .to_tokens(&mut tokens);
                    quote! {
                        impl PartialEq for RuntimeInput {
                            fn eq(&self, other: &Self) -> bool {
                                match (self, other) {
                                    #(#input_eq_arms),*
                                }
                            }
                        }
                    }
                    .to_tokens(&mut tokens);
                    quote! {
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
                    }
                    .to_tokens(&mut tokens);
                }

                // runtime output struct
                {
                    quote! {
                        #[derive(Debug, PartialEq)]
                        pub enum RuntimeOutput {
                            #(#output_variants),*
                        }
                        use RuntimeOutput as O;
                    }
                    .to_tokens(&mut tokens);
                }

                // runtime output struct
                {
                    quote! {
                        #[derive(Debug, Default)]
                        pub struct RuntimeInit {
                            #(#init_fields),*
                        }
                    }
                    .to_tokens(&mut tokens);
                }

                // runtime timer struct
                if !timer_variants.is_empty() {
                    quote! {
                        #[derive(Debug, PartialEq)]
                        pub enum RuntimeTimer {
                            #(#timer_variants),*
                        }
                        use RuntimeTimer as T;
                    }
                    .to_tokens(&mut tokens);

                    quote! {
                        impl timer_stream::Timing for RuntimeTimer {
                            fn get_duration(&self) -> std::time::Duration {
                                match self { #(#timer_duration_arms),* }
                            }
                            fn do_reset(&self) -> bool {
                                match self { #(#timer_reset_arms),* }
                            }
                        }
                    }
                    .to_tokens(&mut tokens);
                }

                // runtime state struct
                {
                    quote! {
                        pub struct Runtime {
                            #(#runtime_fields),*
                        }
                    }
                    .to_tokens(&mut tokens);
                }
            }

            // implementation of `Runtime`
            {
                // `new` runtime function
                let new_runtime = {
                    // initialize services
                    let services_init = self.em.services_handlers.iter().map(
                    |ServiceHandler {
                         service_ident,
                         service_struct_ident,
                         service_mod_ident,
                         ..
                     }| {
                        // parse the function that creates a new runtime
                        let timer = if !timer_variants.is_empty() {
                            quote! {timer.clone()}
                        } else {
                            quote! {}
                        };
                        quote! {
                            let #service_ident = #service_mod_ident::#service_struct_ident::init(
                                output.clone(), #timer
                            );
                        }
                    },
                );
                    // parse the function that creates a new runtime
                    let timer = {
                        if !timer_variants.is_empty() {
                            quote! {timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>}
                        } else {
                            quote! {}
                        }
                    };
                    quote! {
                        pub fn new(output: futures::channel::mpsc::Sender<O>, #timer) -> Runtime {
                            #(#services_init)*
                            Runtime {
                                #(#field_values),*
                            }
                        }
                    }
                };

                // `send_timer` runtime function
                let send_timer = {
                    if !timer_variants.is_empty() {
                        quote! {
                            #[inline]
                            pub async fn send_timer(
                                &mut self, timer: T, instant: std::time::Instant,
                            ) -> Result<(), futures::channel::mpsc::SendError> {
                                self.timer.send((timer, instant)).await?;
                                Ok(())
                            }
                        }
                    } else {
                        quote! {}
                    }
                };

                // `run_loop` function
                let run_loop = self.em.runtime_loop.prepare_tokens(&self.em.input_flows);

                quote! {
                    impl Runtime {
                        #new_runtime

                        #send_timer

                        #run_loop
                    }
                }
                .to_tokens(&mut tokens);
            }

            // services handler functions
            for handler in self.em.services_handlers.iter() {
                handler
                    .prepare_tokens(!timer_variants.is_empty())
                    .to_tokens(&mut tokens)
            }

            tokens
        };

        // the `run` function to be used directly
        let run_fn = {
            // compute channel and stream sizes
            let output_channel_size = self.em.output_flows.len();
            let timer_channel_size = self.em.timing_events.len();
            let prio_stream_size = self.em.input_flows.len() + 1;
            let timer_stream_size = self.em.timing_events.len();

            // output, timer, and priority channels and streams + spawned service
            let (streams, spawn_service);
            if timer_channel_size > 0 {
                streams = {
                    let timer_stream = if self.demo {
                        quote! {
                            const TIMER_CHANNEL_SIZE: usize = #timer_channel_size;
                            const TIMER_STREAM_SIZE: usize = #timer_stream_size;
                            let (timers_sink, timers_stream) = futures::channel::mpsc::channel(TIMER_CHANNEL_SIZE);
                            let timers_stream = timer_stream::timer_stream::<_, _, TIMER_STREAM_SIZE>(timers_stream)
                                .map(|(timer, deadline)| runtime::RuntimeInput::Timer(timer, deadline));
                        }
                    } else {
                        debug_assert!(self.test);
                        quote! {
                            const TIMER_CHANNEL_SIZE: usize = #timer_channel_size;
                            let (timers_sink, timers_stream) = futures::channel::mpsc::channel(TIMER_CHANNEL_SIZE);
                            let timers_stream = timers_stream.map(|(timer, instant): (runtime::RuntimeTimer, std::time::Instant)| {
                                let deadline = instant + timer_stream::Timing::get_duration(&timer);
                                runtime::RuntimeInput::Timer(timer, deadline)
                            });
                        }
                    };
                    let output_stream = quote! {
                        const OUTPUT_CHANNEL_SIZE: usize = #output_channel_size;
                        let (output_sink, output_stream) = futures::channel::mpsc::channel(OUTPUT_CHANNEL_SIZE);
                    };
                    let prio_stream = {
                        let prio_const = if self.demo {
                            quote! { const PRIO_STREAM_SIZE: usize = #prio_stream_size; }
                        } else {
                            debug_assert!(self.test);
                            // TODO: do not use a priority stream but something else
                            quote! { const PRIO_STREAM_SIZE: usize = 100usize; }
                        };
                        quote! {
                            #prio_const
                            let prio_stream = priority_stream::prio_stream::<_, _, PRIO_STREAM_SIZE>(
                                futures::stream::select(input_stream, timers_stream),
                                runtime::RuntimeInput::order,
                            );
                        }
                    };
                    quote! {
                        #timer_stream

                        #output_stream

                        #prio_stream
                    }
                };
                spawn_service = quote! {
                    let service = runtime::Runtime::new(output_sink, timers_sink);
                    tokio::spawn(service.run_loop(INIT, prio_stream, init_signals));
                };
            } else {
                // no timers
                streams = {
                    let output_stream = quote! {
                        const OUTPUT_CHANNEL_SIZE: usize = #output_channel_size;
                        let (output_sink, output_stream) = futures::channel::mpsc::channel(OUTPUT_CHANNEL_SIZE);
                    };
                    let prio_stream = quote! {
                        const PRIO_STREAM_SIZE: usize = #prio_stream_size;
                        let prio_stream = priority_stream::prio_stream::<_, _, PRIO_STREAM_SIZE>(
                            input_stream,
                            runtime::RuntimeInput::order,
                        );
                    };
                    quote! {
                        #output_stream

                        #prio_stream
                    }
                };
                spawn_service = quote! {
                    let service = runtime::Runtime::new(output_sink);
                    tokio::spawn(service.run_loop(INIT, prio_stream, init_signals));
                };
            }

            quote! {
                use futures::{Stream, StreamExt};
                pub fn run(
                    INIT: std::time::Instant,
                    input_stream: impl Stream<Item = runtime::RuntimeInput> + Send + 'static,
                    init_signals: runtime::RuntimeInit,
                ) -> futures::channel::mpsc::Receiver<runtime::RuntimeOutput> {

                    #streams

                    #spawn_service

                    output_stream
                }
            }
        };

        quote! { pub mod runtime { #mod_items } #run_fn }.to_tokens(tokens)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceFlow {
    /// Path of the flow.
    pub path: syn::Path,
    /// The name of the flow.
    pub ident: Ident,
    /// The type of the flow.
    pub typ: Typ,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ArrivingFlow {
    Channel(Ident, Typ, syn::Path),
    Period(Ident),
    Deadline(Ident),
    ServiceDelay(Ident),
    ServiceTimeout(Ident),
}
impl ArrivingFlow {
    pub fn ident(&self) -> &Ident {
        use ArrivingFlow::*;
        match self {
            Channel(id, _, _)
            | Period(id)
            | Deadline(id)
            | ServiceDelay(id)
            | ServiceTimeout(id) => id,
        }
    }
}

/// A timing event structure.
#[derive(Clone, Debug, PartialEq)]
pub struct TimingEvent {
    /// The name of the timing event.
    pub identifier: Ident,
    /// Kind of timing event.
    pub kind: TimingEventKind,
}
#[derive(Clone, Debug, PartialEq)]
pub enum TimingEventKind {
    Period(u64),
    Timeout(u64),
    ServiceTimeout(u64),
    ServiceDelay(u64),
}
