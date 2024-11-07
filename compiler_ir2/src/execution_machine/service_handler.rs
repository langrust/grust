prelude! {
    execution_machine::ArrivingFlow,
}

#[derive(Debug, PartialEq)]
pub struct ServiceHandler {
    /// The service name.
    pub service: String,
    /// Its components.
    pub components: Vec<String>,
    /// The flows handling.
    pub flows_handling: Vec<FlowHandler>,
    /// The signals context from where components will get their inputs.
    pub flows_context: ir1::ctx::Flows,
}

mk_new! { impl ServiceHandler => new {
    service: impl Into<String> = service.into(),
    components: Vec<String>,
    flows_handling: Vec<FlowHandler>,
    flows_context: ir1::ctx::Flows,
} }

impl ServiceHandler {
    /// Transform [ir2] run-loop into an async function performing a loop over events.
    pub fn into_syn(self) -> syn::Item {
        // result
        let mut items = self.flows_context.into_syn().collect::<Vec<_>>();

        // store all inputs in a service_store
        let mut service_store_fields: Vec<syn::Field> = vec![];
        let mut service_store_is_some_s: Vec<syn::Expr> = vec![];
        self.flows_handling.iter().for_each(
            |FlowHandler { arriving_flow, .. }| match arriving_flow {
                ArrivingFlow::Channel(flow_name, flow_type, _) => {
                    let ident = Ident::new(flow_name.as_str(), Span::call_site());
                    let ty = flow_type.into_syn();
                    service_store_fields
                        .push(parse_quote! { #ident: Option<(#ty, std::time::Instant)> });
                    service_store_is_some_s.push(parse_quote! { self.#ident.is_some() });
                }
                ArrivingFlow::Period(time_flow_name) | ArrivingFlow::Deadline(time_flow_name) => {
                    let ident = Ident::new(time_flow_name.as_str(), Span::call_site());
                    service_store_fields
                        .push(parse_quote! { #ident: Option<((), std::time::Instant)> });
                    service_store_is_some_s.push(parse_quote! { self.#ident.is_some() });
                }
                ArrivingFlow::ServiceDelay(_) | ArrivingFlow::ServiceTimeout(_) => (),
            },
        );
        // service store
        let service_store_name = format_ident!(
            "{}",
            to_camel_case(&format!("{}ServiceStore", self.service))
        );
        items.push(syn::Item::Struct(parse_quote! {
            #[derive(Default)]
            pub struct #service_store_name {
                #(#service_store_fields),*
            }
        }));
        // tells is the service_store is not empty
        items.push(syn::Item::Impl(parse_quote! {
            impl #service_store_name {
                pub fn not_empty(&self) -> bool {
                    #(#service_store_is_some_s)||*
                }
            }
        }));

        // create service structure
        let mut service_fields: Vec<syn::Field> = vec![
            parse_quote! { context: Context },
            parse_quote! { delayed: bool },
            parse_quote! { input_store: #service_store_name },
        ];
        let mut field_values: Vec<syn::FieldValue> = vec![
            parse_quote! { context },
            parse_quote! { delayed },
            parse_quote! { input_store },
        ];
        // with components states
        self.components.iter().for_each(|component_name| {
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
        let service_name = format_ident!("{}", to_camel_case(&format!("{}Service", self.service)));
        items.push(syn::Item::Struct(parse_quote! {
            pub struct #service_name {
                #(#service_fields),*
            }
        }));

        // implement the service with `init` and handler functions
        let mut impl_items: Vec<syn::ImplItem> = vec![];

        // create components states
        let components_states = self.components.into_iter().map(|component_name| {
            let component_state_struct =
                format_ident!("{}", to_camel_case(&format!("{}State", component_name)));
            let component_name = format_ident!("{}", component_name);
            let state: syn::Stmt = parse_quote! {
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
        self.flows_handling.into_iter().for_each(
            |FlowHandler {
                 arriving_flow,
                 instruction,
                 ..
             }| {
                let stmts = instruction.into_syn();
                match arriving_flow {
                    ArrivingFlow::Channel(flow_name, flow_type, _) => {
                        let ident = Ident::new(flow_name.as_str(), Span::call_site());
                        let instant = format_ident!("{flow_name}_instant");
                        let function_name: Ident = format_ident!("handle_{flow_name}");
                        let ty = flow_type.into_syn();
                        let message = syn::LitStr::new(
                            format!("{flow_name} changes too frequently").as_str(),
                            Span::call_site(),
                        );
                        impl_items.push(parse_quote! {
                        pub async fn #function_name(
                            &mut self, #instant: std::time::Instant, #ident: #ty
                        ) -> Result<(), futures::channel::mpsc::SendError> {
                            if self.delayed {
                                // reset time constraints
                                self.reset_time_constraints(#instant).await?;
                                // reset all signals' update
                                self.context.reset();
                                // propagate changes
                            #(#stmts)*
                            } else {
                                // store in input_store
                                let unique = self.input_store.#ident.replace((#ident, #instant));
                                assert!(unique.is_none(), #message);
                            }
                            Ok(())
                        }
                    })
                    }
                    ArrivingFlow::Period(time_flow_name)
                    | ArrivingFlow::Deadline(time_flow_name) => {
                        let ident = Ident::new(time_flow_name.as_str(), Span::call_site());
                        let instant = format_ident!("{time_flow_name}_instant");
                        let function_name: Ident = format_ident!("handle_{time_flow_name}");
                        let message = syn::LitStr::new(
                            format!("{time_flow_name} changes too frequently").as_str(),
                            Span::call_site(),
                        );
                        impl_items.push(parse_quote! {
                            pub async fn #function_name(
                                &mut self,  #instant: std::time::Instant
                            ) -> Result<(), futures::channel::mpsc::SendError> {
                                if self.delayed {
                                    // reset time constraints
                                    self.reset_time_constraints(#instant).await?;
                                    // reset all signals' update
                                    self.context.reset();
                                    // propagate changes
                                #(#stmts)*
                                } else {
                                    // store in input_store
                                    let unique = self.input_store.#ident.replace(((), #instant));
                                    assert!(unique.is_none(), #message);
                                }
                                Ok(())
                            }
                        })
                    }
                    ArrivingFlow::ServiceDelay(service_delay) => {
                        let function_name: Ident = format_ident!("handle_{service_delay}");
                        impl_items.push(parse_quote! {
                            pub async fn #function_name(
                                &mut self, instant: std::time::Instant
                            ) -> Result<(), futures::channel::mpsc::SendError> {
                                // reset all signals' update
                                self.context.reset();
                                // propagate changes
                                #(#stmts)*
                                Ok(())
                            }
                        });
                        let enum_ident = Ident::new(
                            to_camel_case(service_delay.as_str()).as_str(),
                            Span::call_site(),
                        );
                        impl_items.push(parse_quote! {
                            #[inline]
                            pub async fn reset_service_delay(
                                &mut self, instant: std::time::Instant
                            ) -> Result<(), futures::channel::mpsc::SendError> {
                                self.timer.send((T::#enum_ident, instant)).await?;
                                Ok(())
                            }
                        })
                    }
                    ArrivingFlow::ServiceTimeout(service_timeout) => {
                        let instant = format_ident!("{service_timeout}_instant");
                        let function_name: Ident = format_ident!("handle_{service_timeout}");
                        impl_items.push(parse_quote! {
                            pub async fn #function_name(
                                &mut self, #instant: std::time::Instant
                            ) -> Result<(), futures::channel::mpsc::SendError> {
                                // reset time constraints
                                self.reset_time_constraints(#instant).await?;
                                // reset all signals' update
                                self.context.reset();
                                // propagate changes
                                #(#stmts)*
                                Ok(())
                            }
                        });
                        let enum_ident = Ident::new(
                            to_camel_case(service_timeout.as_str()).as_str(),
                            Span::call_site(),
                        );
                        impl_items.push(parse_quote! {
                            #[inline]
                            pub async fn reset_service_timeout(
                                &mut self, instant: std::time::Instant
                            ) -> Result<(), futures::channel::mpsc::SendError> {
                                self.timer.send((T::#enum_ident, instant)).await?;
                                Ok(())
                            }
                        })
                    }
                }
            },
        );

        // service handlers in an implementation block
        items.push(syn::Item::Impl(parse_quote! {
            impl #service_name {
                #(#impl_items)*
                #[inline]
                pub async fn reset_time_constraints(
                    &mut self, instant: std::time::Instant
                ) -> Result<(), futures::channel::mpsc::SendError> {
                    self.reset_service_delay(instant).await?;
                    self.reset_service_timeout(instant).await?;
                    self.delayed = false;
                    Ok(())
                }
                #[inline]
                pub async fn send_output(
                    &mut self, output: O
                ) -> Result<(), futures::channel::mpsc::SendError> {
                    self.output.send(output).await?;
                    Ok(())
                }
                #[inline]
                pub async fn send_timer(
                    &mut self, timer: T, instant: std::time::Instant
                ) -> Result<(), futures::channel::mpsc::SendError> {
                    self.timer.send((timer, instant)).await?;
                    Ok(())
                }
            }
        }));

        // service module
        let module_name = format_ident!("{}_service", self.service);
        syn::Item::Mod(parse_quote! {
           pub mod #module_name {
                use futures::{stream::StreamExt, sink::SinkExt};
                use super::*;

                #(#items)*
           }
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct FlowHandler {
    pub arriving_flow: ArrivingFlow,
    pub instruction: FlowInstruction,
}

#[derive(Debug, PartialEq)]
pub enum FlowInstruction {
    Let(String, Expression),
    InitEvent(String),
    UpdateEvent(String, Expression),
    UpdateContext(String, Expression),
    SendSignal(String, Expression, Option<String>),
    SendEvent(String, Expression, Expression, Option<String>),
    IfThrottle(String, String, Constant, Box<Self>),
    IfChange(String, Expression, Box<Self>),
    IfActivated(Vec<String>, Vec<String>, Box<Self>, Option<Box<Self>>),
    ResetTimer(String, String),
    ComponentCall(
        Pattern,
        String,
        Vec<(String, String)>,
        Vec<(String, Option<String>)>,
    ),
    HandleDelay(Vec<String>, Vec<MatchArm>),
    Seq(Vec<Self>),
    Para(BTreeMap<ParaMethod, Vec<Self>>),
}
impl FlowInstruction {
    /// Transform [ir2] instruction on flows into statement.
    pub fn into_syn(self) -> Vec<syn::Stmt> {
        let stmt = match self {
            FlowInstruction::Let(ident, flow_expression) => {
                let ident = Ident::new(&ident, Span::call_site());
                let expression = flow_expression.into_syn();
                parse_quote! { let #ident = #expression; }
            }
            FlowInstruction::InitEvent(ident) => {
                let ident = format_ident!("{}_ref", ident);
                parse_quote! { let #ident = &mut None; }
            }
            FlowInstruction::UpdateEvent(ident, expr) => {
                let ident = format_ident!("{}_ref", ident);
                let expression = expr.into_syn();
                parse_quote! { *#ident = #expression; }
            }
            FlowInstruction::UpdateContext(ident, flow_expression) => {
                let ident = Ident::new(&ident, Span::call_site());
                let expression = flow_expression.into_syn();
                parse_quote! { self.context.#ident.set(#expression); }
            }
            FlowInstruction::SendSignal(name, send_expr, instant) => {
                let enum_ident =
                    Ident::new(to_camel_case(name.as_str()).as_str(), Span::call_site());
                let send_expr = send_expr.into_syn();
                let instant = if let Some(instant) = instant {
                    format_ident!("{instant}_instant")
                } else {
                    Ident::new("instant", Span::call_site())
                };
                parse_quote! { self.send_output(O::#enum_ident(#send_expr, #instant)).await?; }
            }
            FlowInstruction::SendEvent(name, event_expr, send_expr, instant) => {
                let ident = Ident::new(name.as_str(), Span::call_site());
                let enum_ident =
                    Ident::new(to_camel_case(name.as_str()).as_str(), Span::call_site());
                let event_expr = event_expr.into_syn();
                let send_expr = send_expr.into_syn();
                let instant = if let Some(instant) = instant {
                    format_ident!("{instant}_instant")
                } else {
                    Ident::new("instant", Span::call_site())
                };
                parse_quote! {
                    if let Some(#ident) = #event_expr {
                        self.send_output(O::#enum_ident(#send_expr, #instant)).await?;
                    }
                }
            }
            FlowInstruction::IfThrottle(receiver_name, source_name, delta, instruction) => {
                let receiver_ident = Ident::new(&receiver_name, Span::call_site());
                let source_ident = Ident::new(&source_name, Span::call_site());
                let delta = delta.into_syn();
                let instructions = instruction.into_syn();

                parse_quote! {
                    if (self.context.#receiver_ident.get() - #source_ident).abs() >= #delta {
                        #(#instructions)*
                    }
                }
            }
            FlowInstruction::IfChange(old_event_name, signal, then) => {
                let old_event_ident = Ident::new(&old_event_name, Span::call_site());
                let expr = signal.into_syn();
                let then = then.into_syn();
                parse_quote! {
                    if self.context.#old_event_ident.get() != #expr {
                        #(#then)*
                    }
                }
            }
            FlowInstruction::ResetTimer(timer_name, import_name) => {
                let enum_ident = Ident::new(
                    to_camel_case(timer_name.as_str()).as_str(),
                    Span::call_site(),
                );
                let instant = format_ident!("{import_name}_instant");
                parse_quote! { self.send_timer(T::#enum_ident, #instant).await?; }
            }
            FlowInstruction::ComponentCall(
                pattern,
                component_name,
                signals_fields,
                events_fields,
            ) => {
                let outputs = pattern.into_syn();
                let component_ident = Ident::new(&component_name, Span::call_site());
                let component_input_name =
                    format_ident!("{}", to_camel_case(&format!("{component_name}Input")));

                let input_fields = signals_fields
                    .into_iter()
                    .map(|(field_name, in_context)| -> syn::FieldValue {
                        let field_id = Ident::new(&field_name, Span::call_site());
                        let in_context_id = Ident::new(&in_context, Span::call_site());
                        let expr: syn::Expr = parse_quote!(self.context.#in_context_id.get());
                        parse_quote! { #field_id : #expr }
                    })
                    .chain(events_fields.into_iter().map(|(field_name, opt_event)| {
                        let field_id = Ident::new(&field_name, Span::call_site());
                        if let Some(event_name) = opt_event {
                            let event_id = format_ident!("{event_name}_ref");
                            parse_quote! { #field_id : *#event_id }
                        } else {
                            parse_quote! { #field_id : None }
                        }
                    }));

                parse_quote! {
                    let #outputs = self.#component_ident.step(
                        #component_input_name {
                            #(#input_fields),*
                        }
                    );
                }
            }
            FlowInstruction::HandleDelay(input_flows, match_arms) => {
                let input_flows = input_flows.iter().map(|name| -> syn::Expr {
                    let ident = Ident::new(name, Span::call_site());
                    parse_quote! { self.input_store.#ident.take() }
                });
                let arms = match_arms.into_iter().map(|arm| arm.into_syn());
                parse_quote! {
                    if self.input_store.not_empty() {
                        self.reset_time_constraints(instant).await?;
                        match (#(#input_flows),*) {
                            #(#arms)*
                        }
                    } else {
                        self.delayed = true;
                    }
                }
            }
            FlowInstruction::IfActivated(events, signals, then, els) => {
                let activation_cond = events
                    .iter()
                    .map(|e| -> syn::Expr {
                        let ident = format_ident!("{e}_ref");
                        parse_quote! { #ident.is_some() }
                    })
                    .chain(signals.iter().map(|s| -> syn::Expr {
                        let ident = Ident::new(s, Span::call_site());
                        parse_quote! { self.context.#ident.is_new() }
                    }));
                let then_instrs = then.into_syn();

                if events.is_empty() && signals.is_empty() {
                    return els.map_or(vec![], |instr| instr.into_syn());
                } else {
                    if let Some(instr) = els {
                        let els_instrs = instr.into_syn();
                        parse_quote! {
                            if #(#activation_cond)||* {
                                #(#then_instrs)*
                            } else {
                                #(#els_instrs)*
                            }
                        }
                    } else {
                        parse_quote! {
                            if #(#activation_cond)||* {
                                #(#then_instrs)*
                            }
                        }
                    }
                }
            }
            FlowInstruction::Seq(instrs) => {
                return instrs
                    .into_iter()
                    .flat_map(|instr| instr.into_syn())
                    .collect()
            }
            FlowInstruction::Para(method_map) => {
                let para_futures = method_map.into_iter().flat_map(|(_method, para_instrs)| {
                    para_instrs.into_iter().map(|instr| -> syn::Expr {
                        let stmts = instr.into_syn();
                        parse_quote! {async { #(#stmts)* }}
                    })
                });
                parse_quote! {
                    tokio::join!(#(#para_futures),*);
                }
            }
        };
        vec![stmt]
    }

    pub fn send(name: impl Into<String> + Copy, expr: Expression, is_event: bool) -> Self {
        if is_event {
            FlowInstruction::SendEvent(
                name.into(),
                Expression::event(name.into()).into(),
                expr.into(),
                None,
            )
        } else {
            FlowInstruction::SendSignal(name.into(), expr.into(), None)
        }
    }
    pub fn send_from(
        name: impl Into<String> + Copy,
        expr: Expression,
        instant: impl Into<String>,
        is_event: bool,
    ) -> Self {
        if is_event {
            FlowInstruction::SendEvent(
                name.into(),
                Expression::event(name.into()).into(),
                expr.into(),
                Some(instant.into()),
            )
        } else {
            FlowInstruction::SendSignal(name.into(), expr.into(), Some(instant.into()))
        }
    }
}
mk_new! { impl FlowInstruction =>
    Let: def_let (
        name: impl Into<String> = name.into(),
        expr: Expression = expr.into(),
    )
    InitEvent: init_event (
        name: impl Into<String> = name.into(),
    )
    UpdateEvent: update_event (
        name: impl Into<String> = name.into(),
        expr: Expression = expr.into(),
    )
    UpdateContext: update_ctx (
        name: impl Into<String> = name.into(),
        expr: Expression = expr.into(),
    )
    IfThrottle: if_throttle (
        flow_name: impl Into<String> = flow_name.into(),
        source_name: impl Into<String> = source_name.into(),
        delta: Constant = delta,
        instr: FlowInstruction = instr.into(),
    )
    IfChange: if_change (
        old_event_name: impl Into<String> = old_event_name.into(),
        signal: Expression = signal,
        then: FlowInstruction = then.into(),
    )
    IfActivated: if_activated (
        events: impl Into<Vec<String>> = events.into(),
        signals: impl Into<Vec<String>> = signals.into(),
        then: FlowInstruction = then.into(),
        els: Option<FlowInstruction> = els.map(Into::into),
    )
    ResetTimer: reset (
        name: impl Into<String> = name.into(),
        instant: impl Into<String> = instant.into(),
    )
    ComponentCall: comp_call (
        pat: Pattern = pat,
        name: impl Into<String> = name.into(),
        signals: impl Into<Vec<(String, String)>> = signals.into(),
        events: impl Into<Vec<(String, Option<String>)>> = events.into(),
    )
    HandleDelay: handle_delay(
        input_names: impl Iterator<Item = String> = input_names.collect(),
        arms: impl Iterator<Item = MatchArm> = arms.collect(),
    )
    Seq: seq(
        instrs: Vec<FlowInstruction> = instrs,
    )
    Para: para(
        para_instr: BTreeMap<ParaMethod, Vec<Self>> = para_instr,
    )
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParaMethod {
    Rayon,
    Threads,
    Tokio,
    DoNotPara,
}
mk_new! { impl ParaMethod =>
    Rayon: rayon ()
    Threads: threads ()
    Tokio: tokio ()
    DoNotPara: no_para ()
}

#[derive(Debug, PartialEq)]
pub struct MatchArm {
    pub patterns: Vec<Pattern>,
    pub instr: FlowInstruction,
}
mk_new! { impl MatchArm =>
    new {
        patterns: Vec<Pattern> = patterns,
        instr: FlowInstruction = instr,
    }
}

impl MatchArm {
    fn into_syn(self) -> syn::Arm {
        let syn_pats = self.patterns.into_iter().map(|pat| pat.into_syn());
        let stmts = self.instr.into_syn();
        parse_quote! {
            (#(#syn_pats),*) => {
                #(#stmts)*
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    /// A literal expression: `1` or `"hello world"`.
    Literal {
        /// The literal.
        literal: Constant,
    },
    /// An event call: `x`.
    Event {
        /// The identifier.
        identifier: String,
    },
    /// An identifier call: `x`.
    Identifier {
        /// The identifier.
        identifier: String,
    },
    /// A call from the context: `ctx.s`.
    InContext {
        /// The flow called.
        flow: String,
    },
    /// A call from the context that will take the value: `ctx.s.take()`.
    TakeFromContext {
        /// The flow called.
        flow: String,
    },
    /// Some expression: `Some(v)`.
    Some {
        /// The value expression inside.
        expression: Box<Expression>,
    },
    /// None expression: `None`.
    None,
}

mk_new! { impl Expression =>
    Literal: lit {
        literal: Constant = literal
    }
    Event: event {
        identifier: impl Into<String> = identifier.into()
    }
    Identifier: ident {
        identifier: impl Into<String> = identifier.into()
    }
    InContext: in_ctx {
        flow: impl Into<String> = flow.into()
    }
    TakeFromContext: take_from_ctx {
        flow: impl Into<String> = flow.into()
    }
    Some: some {
        expression: Expression = expression.into()
    }
    None: none {}
}

impl Expression {
    pub fn into_syn(self) -> syn::Expr {
        match self {
            Expression::Literal { literal } => literal.into_syn(),
            Expression::Event { identifier } => {
                let identifier = format_ident!("{}_ref", identifier);
                parse_quote! { *#identifier }
            }
            Expression::Identifier { identifier } => {
                let identifier = Ident::new(&identifier, Span::call_site());
                parse_quote! { #identifier }
            }
            Expression::InContext { flow } => {
                let flow = Ident::new(&flow, Span::call_site());
                parse_quote! { self.context.#flow.get() }
            }
            Expression::TakeFromContext { flow } => {
                let flow = Ident::new(&flow, Span::call_site());
                parse_quote! { std::mem::take(&mut self.context.#flow.0) }
            }
            Expression::Some { expression } => {
                let expression = expression.into_syn();
                parse_quote! { Some(#expression) }
            }
            Expression::None => parse_quote! { None },
        }
    }
}
