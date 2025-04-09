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

impl ExecutionMachine {
    /// Transform [ir2] execution-machine into a runtime module.
    pub fn into_syn(self, mut stats: StatsMut) -> syn::Item {
        let (mut runtime_items, field_values) = {
            let mut runtime_items: Vec<syn::Item> = vec![
                parse_quote! { use futures::{stream::StreamExt, sink::SinkExt}; },
                parse_quote! { use super::*; },
                parse_quote! { use RuntimeTimer as T; },
                parse_quote! { use RuntimeInput as I; },
                parse_quote! { use RuntimeOutput as O; },
            ];

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

            stats.timed("timing events", || {
                for TimingEvent { identifier, kind } in self.timing_events.iter() {
                    let enum_ident = identifier.to_camel();
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
            });

            stats.timed("interface, input flows", || {
                for InterfaceFlow {
                    identifier, typ, ..
                } in self.input_flows.iter()
                {
                    let enum_ident = identifier.to_camel();
                    let ty = typ.into_syn();
                    input_variants.push(parse_quote! { #enum_ident(#ty, std::time::Instant) });
                    input_eq_arms.push(parse_quote! {
                        (I::#enum_ident(this, _), I::#enum_ident(other, _)) => this.eq(other)
                    });
                    let instant = Ident::instant_var();
                    input_get_instant_arms
                        .push(parse_quote! { I::#enum_ident(_, #instant) => *#instant });
                }
            });

            stats.timed("interface, output flows", || {
                for InterfaceFlow {
                    identifier, typ, ..
                } in self.output_flows.iter()
                {
                    let enum_ident = identifier.to_camel();
                    let ty = typ.into_syn();
                    output_variants.push(parse_quote! { #enum_ident(#ty, std::time::Instant) });
                }
            });

            if !timer_variants.is_empty() {
                input_variants.push(parse_quote! { Timer(T, std::time::Instant) });
                input_eq_arms.push(
                    parse_quote! { (I::Timer(this, _), I::Timer(other, _)) => this.eq(other) },
                );
                let instant = Ident::instant_var();
                input_get_instant_arms.push(parse_quote! { I::Timer(_, #instant) => *#instant });
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
            self.services_handlers.iter().for_each(
                |ServiceHandler {
                     service_ident,
                     service_mod_ident,
                     service_struct_ident,
                     ..
                 }| {
                    runtime_fields.push(parse_quote! {
                        #service_ident: #service_mod_ident::#service_struct_ident
                    });
                    field_values.push(parse_quote! { #service_ident });
                },
            );
            runtime_fields.push(parse_quote! { output: futures::channel::mpsc::Sender<O> });
            field_values.push(parse_quote! { output });
            runtime_fields.push(
                parse_quote! { timer: futures::channel::mpsc::Sender<(T, std::time::Instant)> },
            );
            field_values.push(parse_quote! { timer });

            runtime_items.push(syn::Item::Struct(parse_quote! {
                pub struct Runtime {
                    #(#runtime_fields),*
                }
            }));

            (runtime_items, field_values)
        };

        // create a new runtime
        let new_runtime = stats.timed("runtime creation", || {
            // initialize services
            let services_init = self.services_handlers.iter().map(
                |ServiceHandler {
                     service_ident,
                     service_struct_ident,
                     service_mod_ident,
                     ..
                 }| {
                    let state: syn::Stmt = parse_quote! {
                        let #service_ident = #service_mod_ident::#service_struct_ident::init(
                            output.clone(), timer.clone()
                        );
                    };
                    state
                },
            );
            // parse the function that creates a new runtime
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
        });

        // create the runtime loop
        let run_loop = self.runtime_loop.into_syn(self.output_flows);

        let impl_runtime = syn::ItemImpl::new_simple(
            parse_quote!(Runtime),
            vec![
                new_runtime,
                parse_quote! {
                    #[inline]
                    pub async fn send_output(
                        &mut self, output: O
                    ) -> Result<(), futures::channel::mpsc::SendError> {
                        self.output.send(output).await?;
                        Ok(())
                    }
                },
                parse_quote! {
                    #[inline]
                    pub async fn send_timer(
                        &mut self, timer: T, instant: std::time::Instant
                    ) -> Result<(), futures::channel::mpsc::SendError> {
                        self.timer.send((timer, instant)).await?;
                        Ok(())
                    }
                },
                run_loop,
            ],
        );

        runtime_items.push(syn::Item::Impl(impl_runtime));

        // create the services handlers
        stats.timed_with(
            format!(
                "service handlers creation ({})",
                self.services_handlers.len()
            ),
            |mut stats| {
                for handler in self.services_handlers.into_iter() {
                    runtime_items.push(handler.into_syn(&mut stats));
                }
            },
        );

        // parse the runtime module
        stats.timed("parse-quote runtime module", || {
            syn::Item::Mod(syn::ItemMod::new_simple(
                parse_quote!(runtime),
                runtime_items,
            ))
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceFlow {
    /// Path of the flow.
    pub path: syn::Path,
    /// The name of the flow.
    pub identifier: Ident,
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
