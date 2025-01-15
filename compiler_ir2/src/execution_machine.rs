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

            stats.timed("timing events", || {
                for TimingEvent { identifier, kind } in self.timing_events.iter() {
                    let enum_ident = Ident::new(
                        to_camel_case(&identifier.to_string()).as_str(),
                        identifier.span(),
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
            });

            stats.timed("interface, input flows", || {
                for InterfaceFlow {
                    identifier, typ, ..
                } in self.input_flows.iter()
                {
                    let enum_ident =
                        Ident::new(&to_camel_case(&identifier.to_string()), identifier.span());
                    let ty = typ.into_syn();
                    input_variants.push(parse_quote! { #enum_ident(#ty, std::time::Instant) });
                    input_eq_arms.push(parse_quote! {
                        (I::#enum_ident(this, _), I::#enum_ident(other, _)) => this.eq(other)
                    });
                    input_get_instant_arms
                        .push(parse_quote! { I::#enum_ident(_, instant) => *instant });
                }
            });

            stats.timed("interface, output flows", || {
                for InterfaceFlow {
                    identifier, typ, ..
                } in self.output_flows.into_iter()
                {
                    let enum_ident =
                        Ident::new(&to_camel_case(&identifier.to_string()), identifier.span());
                    let ty = typ.into_syn();
                    output_variants.push(parse_quote! { #enum_ident(#ty, std::time::Instant) });
                }
            });

            if !timer_variants.is_empty() {
                input_variants.push(parse_quote! { Timer(T, std::time::Instant) });
                input_eq_arms.push(
                    parse_quote! { (I::Timer(this, _), I::Timer(other, _)) => this.eq(other) },
                );
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
            self.services_handlers.iter().for_each(|service_handler| {
                let service_name = &service_handler.service;
                let service_path = format_ident!("{}_service", service_name);
                let service_state_struct =
                    format_ident!("{}", to_camel_case(&format!("{}Service", service_name)));
                let service_name = format_ident!("{}", service_name);
                runtime_fields.push(parse_quote! {
                    #service_name: #service_path::#service_state_struct
                });
                field_values.push(parse_quote! { #service_name });
            });
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
            let nb_services = self.services_handlers.len();
            let is_last = |idx| idx < nb_services - 1;
            // initialize services
            let services_init =
                self.services_handlers
                    .iter()
                    .enumerate()
                    .map(|(idx, service_handler)| {
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
                            let #service_name = #service_path::#service_state_struct::init(
                                #output_channel, timer.clone()
                            );
                        };
                        state
                    });
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
        let run_loop = self.runtime_loop.into_syn();

        // create the services handlers
        let handlers = stats.timed_with(
            format!(
                "service handlers creation ({})",
                self.services_handlers.len()
            ),
            |mut stats| {
                self.services_handlers
                    .into_iter()
                    .map(|handler| handler.into_syn(&mut stats))
                    .collect_vec()
            },
        );

        // parse the runtime module
        stats.timed("parse-quote runtime module", || {
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
                        pub async fn send_timer(
                            &mut self, timer: T, instant: std::time::Instant
                        ) -> Result<(), futures::channel::mpsc::SendError> {
                            self.timer.send((timer, instant)).await?;
                            Ok(())
                        }

                        #run_loop
                    }

                    #(#handlers)*
                }
            })
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
