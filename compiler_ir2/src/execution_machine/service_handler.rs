prelude! {
    execution_machine::ArrivingFlow,
}

#[derive(Debug, PartialEq)]
pub struct ComponentInfo {
    pub ty_ident: Ident,
    pub state_ty_ident: Ident,
    pub field_ident: Ident,
}
impl ComponentInfo {
    pub fn new(mem_name: Ident, ty_ident: Ident) -> Self {
        let field_ident = mem_name.to_field();
        let state_ty_ident = ty_ident.to_state_ty();
        Self {
            ty_ident,
            field_ident,
            state_ty_ident,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ServiceHandler {
    /// The service name.
    pub service_ident: Ident,
    /// The flows handling.
    pub flow_handlers: Vec<FlowHandler>,
    /// The signals context from where components will get their inputs.
    pub flow_context: ir1::ctx::Flows,

    pub components_info: Vec<ComponentInfo>,
    pub service_store_ident: Ident,
    pub service_struct_ident: Ident,
    pub service_mod_ident: Ident,
}

impl ServiceHandler {
    pub fn new(
        service: impl Into<Ident>,
        components: Vec<(Ident, Ident)>,
        flow_handlers: Vec<FlowHandler>,
        flow_context: ir1::ctx::Flows,
    ) -> Self {
        let service = service.into();
        let components_info = components
            .iter()
            .map(|(mem_name, ty_ident)| ComponentInfo::new(mem_name.clone(), ty_ident.clone()))
            .collect();
        let service_store_ident = service.to_service_store_ty();
        let service_struct_ident = service.to_service_state_ty();
        let service_mod_ident = service.to_service_mod();
        Self {
            service_ident: service,
            flow_handlers,
            flow_context,
            components_info,
            service_store_ident,
            service_struct_ident,
            service_mod_ident,
        }
    }

    /// Transform [ir2] run-loop into an async function performing a loop over events.
    pub fn into_syn(self, stats: &mut StatsMut) -> syn::Item {
        // result
        let mut items = self.flow_context.into_syn();

        // store all inputs in a service_store
        let stitem = stats.start(format!(
            "store inputs in `service_store` ({})",
            self.flow_handlers.len()
        ));
        let mut service_store_fields: Vec<syn::Field> = vec![];
        let mut service_store_is_some_s: Vec<syn::Expr> = vec![];
        self.flow_handlers.iter().for_each(
            |FlowHandler { arriving_flow, .. }| match arriving_flow {
                ArrivingFlow::Channel(flow_name, flow_type, _) => {
                    let ident = flow_name;
                    let ty = flow_type.into_syn();
                    service_store_fields
                        .push(parse_quote! { #ident: Option<(#ty, std::time::Instant)> });
                    service_store_is_some_s.push(parse_quote! { self.#ident.is_some() });
                }
                ArrivingFlow::Period(time_flow_name) | ArrivingFlow::Deadline(time_flow_name) => {
                    let ident = time_flow_name;
                    service_store_fields
                        .push(parse_quote! { #ident: Option<((), std::time::Instant)> });
                    service_store_is_some_s.push(parse_quote! { self.#ident.is_some() });
                }
                ArrivingFlow::ServiceDelay(_) | ArrivingFlow::ServiceTimeout(_) => (),
            },
        );
        // service store
        let service_store_ident = &self.service_store_ident;
        items.push(syn::Item::Struct(parse_quote! {
            #[derive(Default)]
            pub struct #service_store_ident {
                #(#service_store_fields),*
            }
        }));
        // tells is the service_store is not empty
        items.push(syn::Item::Impl(parse_quote! {
            impl #service_store_ident {
                pub fn not_empty(&self) -> bool {
                    #(#service_store_is_some_s)||*
                }
            }
        }));
        stats.augment_end(stitem);

        // create service structure
        let stitem = stats.start("create service structure");
        let mut service_fields: Vec<syn::Field> = vec![
            parse_quote! { begin: std::time::Instant },
            parse_quote! { context: Context },
            parse_quote! { delayed: bool },
            parse_quote! { input_store: #service_store_ident },
        ];
        let mut field_values: Vec<syn::FieldValue> = vec![
            parse_quote! { begin: std::time::Instant::now() },
            parse_quote! { context },
            parse_quote! { delayed },
            parse_quote! { input_store },
        ];
        // with components states
        self.components_info.iter().for_each(
            |ComponentInfo {
                 state_ty_ident,
                 field_ident,
                 ..
             }| {
                service_fields.push(parse_quote! {
                    #field_ident: #state_ty_ident
                });
                field_values.push(parse_quote! { #field_ident });
            },
        );
        // and sending channels
        service_fields.push(parse_quote! { output: futures::channel::mpsc::Sender<O> });
        service_fields
            .push(parse_quote! { timer: futures::channel::mpsc::Sender<(T, std::time::Instant)> });
        field_values.push(parse_quote! { output });
        field_values.push(parse_quote! { timer });
        let service_name = &self.service_struct_ident;
        items.push(syn::Item::Struct(parse_quote! {
            pub struct #service_name {
                #(#service_fields),*
            }
        }));
        stats.augment_end(stitem);

        // implement the service with `init` and handler functions
        let mut impl_items: Vec<syn::ImplItem> = vec![];

        // create components states
        let stitem = stats.start("component states");
        let components_states = self.components_info.into_iter().map(
            |ComponentInfo {
                 state_ty_ident,
                 field_ident,
                 ..
             }| {
                let state: syn::Stmt = parse_quote! {
                    let #field_ident = <#state_ty_ident as grust::core::Component>::init();
                };
                state
            },
        );
        stats.augment_end(stitem);

        // `init` function
        let stitem = stats.start("`init` function");
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
        stats.augment_end(stitem);

        // flows handler functions
        stats.timed_with(
            format!("flow handler functions ({})", self.flow_handlers.len()),
            |mut stats| {
                self.flow_handlers.into_iter().for_each(
                    |FlowHandler {
                         arriving_flow,
                         instruction,
                         ..
                     }| {
                        // let stmts = stats.timed_with(
                        //     format!("instruction to syn ({})", arriving_flow.ident()),
                        //     |stats| instruction.into_syn2(stats),
                        // );
                        let stmts = instruction.into_syn2(stats.as_mut());
                        match arriving_flow {
                            ArrivingFlow::Channel(flow_name, flow_type, _) => {
                                let instant = flow_name.to_instant_var();
                                let function_name: Ident = flow_name.to_handle_fn();
                                let ty = flow_type.into_syn();
                                let message = syn::LitStr::new(
                                    format!("flow `{flow_name}` changes too frequently").as_str(),
                                    Span::call_site(),
                                );
                                impl_items.push(parse_quote! {
                                    pub async fn #function_name(
                                        &mut self, #instant: std::time::Instant, #flow_name: #ty
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
                                            let unique =
                                                self.input_store.#flow_name
                                                    .replace((#flow_name, #instant));
                                            assert!(unique.is_none(), #message);
                                        }
                                        Ok(())
                                    }
                                })
                            }
                            ArrivingFlow::Period(time_flow_name)
                            | ArrivingFlow::Deadline(time_flow_name) => {
                                let instant = time_flow_name.to_instant_var();
                                let function_name: Ident = time_flow_name.to_handle_fn();
                                let message = syn::LitStr::new(
                                    format!("flow `{time_flow_name}` changes too frequently")
                                        .as_str(),
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
                                            let unique =
                                                self.input_store.#time_flow_name
                                                    .replace(((), #instant));
                                            assert!(unique.is_none(), #message);
                                        }
                                        Ok(())
                                    }
                                })
                            }
                            ArrivingFlow::ServiceDelay(service_delay) => {
                                let instant = Ident::instant_var();
                                let function_name = service_delay.to_handle_fn();
                                impl_items.push(parse_quote! {
                                    pub async fn #function_name(
                                        &mut self, #instant: std::time::Instant
                                    ) -> Result<(), futures::channel::mpsc::SendError> {
                                        // reset all signals' update
                                        self.context.reset();
                                        // propagate changes
                                        #(#stmts)*
                                        Ok(())
                                    }
                                });
                                let enum_ident = service_delay.to_ty();
                                impl_items.push(parse_quote! {
                                    #[inline]
                                    pub async fn reset_service_delay(
                                        &mut self, #instant: std::time::Instant
                                    ) -> Result<(), futures::channel::mpsc::SendError> {
                                        self.timer.send((T::#enum_ident, #instant)).await?;
                                        Ok(())
                                    }
                                })
                            }
                            ArrivingFlow::ServiceTimeout(service_timeout) => {
                                let instant = service_timeout.to_instant_var();
                                let function_name = service_timeout.to_handle_fn();
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
                                let enum_ident = service_timeout.to_ty();
                                impl_items.push(parse_quote! {
                                    #[inline]
                                    pub async fn reset_service_timeout(
                                        &mut self, #instant: std::time::Instant
                                    ) -> Result<(), futures::channel::mpsc::SendError> {
                                        self.timer.send((T::#enum_ident, #instant)).await?;
                                        Ok(())
                                    }
                                })
                            }
                        }
                    },
                )
            },
        );

        // service handlers in an implementation block
        let stitem = stats.start("service handlers block");
        items.push(syn::Item::Impl(parse_quote! {
            impl #service_name {
                #(#impl_items)*
                #[inline]
                pub async fn reset_time_constraints(
                    &mut self, instant: std::time::Instant
                ) -> Result<(), futures::channel::mpsc::SendError> {
                    self.reset_service_delay(instant).await?;
                    self.delayed = false;
                    Ok(())
                }
                #[inline]
                pub async fn send_output(
                    &mut self, output: O, instant: std::time::Instant
                ) -> Result<(), futures::channel::mpsc::SendError> {
                    self.reset_service_timeout(instant).await?;
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
        stats.augment_end(stitem);

        // service module
        let module_name = &self.service_mod_ident;
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
    Let(Ident, Expression),
    InitEvent(Ident),
    UpdateEvent(Ident, Expression),
    UpdateContext(Ident, Expression),
    SendSignal(Ident, Expression, Option<Ident>),
    SendEvent(Ident, Expression, Expression, Option<Ident>),
    IfThrottle(Ident, Ident, Constant, Box<Self>),
    IfChange(Ident, Expression, Box<Self>),
    IfActivated(Vec<Ident>, Vec<Ident>, Box<Self>, Option<Box<Self>>),
    ResetTimer(Ident, Ident),
    ComponentCall(Pattern, Ident, Ident, Vec<(Ident, Expression)>),
    FunctionCall(Pattern, Ident, Vec<Expression>),
    HandleDelay(Vec<Ident>, Vec<MatchArm>),
    Seq(Vec<Self>),
    Para(BTreeMap<ParaMethod, Vec<Self>>),
}
impl FlowInstruction {
    pub fn seq(mut vec: Vec<Self>) -> Self {
        if vec.len() == 1 {
            vec.pop().expect("len is `1`")
        } else {
            Self::Seq(vec)
        }
    }

    pub fn into_syn2(self, stats: StatsMut) -> Vec<syn::Stmt> {
        let mut vec = vec![];
        self.into_syn_mut(&mut vec, stats);
        vec
    }

    /// Transform [ir2] instruction on flows into statement.
    pub fn into_syn_mut(self, vec: &mut Vec<syn::Stmt>, mut stats: StatsMut) {
        let stmt = match self {
            FlowInstruction::Let(ident, flow_expression) => stats.timed("let", || {
                let expression = flow_expression.into_syn();
                parse_quote! { let #ident = #expression; }
            }),
            FlowInstruction::InitEvent(ident) => stats.timed("init event", || {
                let ident = ident.to_ref_var();
                parse_quote! { let #ident = &mut None; }
            }),
            FlowInstruction::UpdateEvent(ident, expr) => stats.timed("update event", || {
                let ident = ident.to_ref_var();
                let expression = expr.into_syn();
                parse_quote! { *#ident = #expression; }
            }),
            FlowInstruction::UpdateContext(ident, flow_expression) => {
                stats.timed("update context", || {
                    let expression = flow_expression.into_syn();
                    parse_quote! { self.context.#ident.set(#expression); }
                })
            }
            FlowInstruction::SendSignal(name, send_expr, instant) => {
                stats.timed("send signal", || {
                    let enum_ident = name.to_camel();
                    let send_expr = send_expr.into_syn();
                    let instant = if let Some(instant) = instant {
                        instant.to_instant_var()
                    } else {
                        Ident::instant_var()
                    };
                    parse_quote! { self.send_output(O::#enum_ident(#send_expr, #instant), #instant).await?; }
                })
            }
            FlowInstruction::SendEvent(name, event_expr, send_expr, instant) => {
                stats.timed("send event", || {
                    let enum_ident = name.to_camel();
                    let event_expr = event_expr.into_syn();
                    let send_expr = send_expr.into_syn();
                    let instant = if let Some(instant) = instant {
                        instant.to_instant_var()
                    } else {
                        Ident::instant_var()
                    };
                    parse_quote! {
                        if let Some(#name) = #event_expr {
                            self.send_output(O::#enum_ident(#send_expr, #instant), #instant).await?;
                        }
                    }
                })
            }
            FlowInstruction::IfThrottle(receiver_name, source_name, delta, instruction) => {
                let receiver_ident = receiver_name;
                let source_ident = source_name;
                let delta = delta.into_syn();
                let mut instructions = vec![];
                stats.augment_timed_with("sub if throttle", |stats| {
                    instruction.into_syn_mut(&mut instructions, stats)
                });

                parse_quote! {
                    if (self.context.#receiver_ident.get() - #source_ident).abs() >= #delta {
                        #(#instructions)*
                    }
                }
            }
            FlowInstruction::IfChange(old_event_name, signal, then) => {
                let old_event_ident = old_event_name;
                let expr = signal.into_syn();
                let mut thens = vec![];
                stats.augment_timed_with("then branch in if change", |stats| {
                    then.into_syn_mut(&mut thens, stats)
                });
                parse_quote! {
                    if self.context.#old_event_ident.get() != #expr {
                        #(#thens)*
                    }
                }
            }
            FlowInstruction::ResetTimer(timer_name, import_name) => {
                let enum_ident = timer_name.to_ty();
                let instant = import_name.to_instant_var();
                parse_quote! { self.send_timer(T::#enum_ident, #instant).await?; }
            }
            FlowInstruction::ComponentCall(pattern, memory_name, comp_name, inputs_fields) => {
                let outputs = pattern.into_syn();
                let mem_ident = memory_name.to_field();
                let input_ty = comp_name.to_input_ty();
                let state_ty = comp_name.to_state_ty();

                let input_fields =
                    inputs_fields
                        .into_iter()
                        .map(|(field_name, input)| -> syn::FieldValue {
                            let field_id = field_name;
                            let expr: syn::Expr = input.into_syn();
                            parse_quote! { #field_id : #expr }
                        });

                parse_quote! {
                    let #outputs = <#state_ty as grust::core::Component>::step(
                        &mut self.#mem_ident,
                        #input_ty {
                            #(#input_fields),*
                        }
                    );
                }
            }
            FlowInstruction::FunctionCall(pattern, function_name, inputs) => {
                let outputs = pattern.into_syn();
                let function_ident = function_name.to_field();

                let inputs =
                    inputs
                        .into_iter()
                        .map(|input| input.into_syn());

                parse_quote! {
                    let #outputs = self.#function_ident(#(#inputs),*);
                }
            }
            FlowInstruction::HandleDelay(input_flows, match_arms) => {
                stats.timed_with("handle delay", |mut stats| {
                    let input_flows = input_flows.iter().map(|name| -> syn::Expr {
                        let ident = name;
                        parse_quote! { self.input_store.#ident.take() }
                    });
                    let arms = match_arms
                        .into_iter()
                        .map(|arm| arm.into_syn(stats.as_mut()));
                    let instant = Ident::instant_var();
                    parse_quote! {
                        if self.input_store.not_empty() {
                            self.reset_time_constraints(#instant).await?;
                            match (#(#input_flows),*) {
                                #(#arms)*
                            }
                        } else {
                            self.delayed = true;
                        }
                    }
                })
            }
            FlowInstruction::IfActivated(events, signals, then, els) => {
                let stitem = stats.start("if activated");
                {
                    let activation_cond = events
                        .iter()
                        .map(|e| -> syn::Expr {
                            let ident = e.to_ref_var();
                            parse_quote! { #ident.is_some() }
                        })
                        .chain(signals.iter().map(|s| -> syn::Expr {
                            let ident = s;
                            parse_quote! { self.context.#ident.is_new() }
                        }));
                    let mut then_instrs = vec![];
                    then.into_syn_mut(&mut then_instrs, stats.as_mut());

                    if events.is_empty() && signals.is_empty() {
                        if let Some(els) = els {
                            els.into_syn_mut(vec, stats.as_mut());
                        }
                        stats.augment_end(stitem);
                        return ();
                    } else {
                        if let Some(instr) = els {
                            let mut els_instrs = vec![];
                            // stats.augment_timed_with("els in if activated", |stats| {
                            instr.into_syn_mut(&mut els_instrs, stats.as_mut());
                            // });
                            stats.augment_end(stitem);
                            parse_quote! {
                                if #(#activation_cond)||* {
                                    #(#then_instrs)*
                                } else {
                                    #(#els_instrs)*
                                }
                            }
                        } else {
                            stats.augment_end(stitem);
                            parse_quote! {
                                if #(#activation_cond)||* {
                                    #(#then_instrs)*
                                }
                            }
                        }
                    }
                }
            }
            FlowInstruction::Seq(instrs) => {
                // stats.augment_timed_with(
                //     format!("instruction in seq ({})", instrs.len()),
                //     |mut stats| {
                let mut stack = vec![instrs.into_iter()];
                while let Some(mut iter) = stack.pop() {
                    if let Some(instr) = iter.next() {
                        stack.push(iter);
                        if let Self::Seq(subs) = instr {
                            stack.push(subs.into_iter());
                            continue;
                        } else {
                            instr.into_syn_mut(vec, stats.as_mut())
                        }
                    }
                }
                //     },
                // );
                return ();
            }
            FlowInstruction::Para(method_map) => {
                let stats = &mut stats;
                let para_futures = method_map.into_iter().flat_map(|(_method, para_instrs)| {
                    para_instrs
                        .into_iter()
                        .map(|instr| -> syn::Expr {
                            let stmts = stats.augment_timed_with("para statements", |stats| {
                                instr.into_syn2(stats)
                            });
                            parse_quote! {async { #(#stmts)* }}
                        })
                        .collect::<Vec<_>>()
                });
                parse_quote! {
                    tokio::join!(#(#para_futures),*);
                }
            }
        };
        vec.push(stmt)
    }

    pub fn send(name: impl Into<Ident>, expr: Expression, is_event: bool) -> Self {
        let name = name.into();
        if is_event {
            FlowInstruction::SendEvent(name.clone(), Expression::event(name).into(), expr, None)
        } else {
            FlowInstruction::SendSignal(name, expr, None)
        }
    }
    pub fn send_from(
        name: impl Into<Ident>,
        expr: Expression,
        instant: impl Into<Ident>,
        is_event: bool,
    ) -> Self {
        let (name, instant) = (name.into(), instant.into());
        if is_event {
            FlowInstruction::SendEvent(name.clone(), Expression::event(name), expr, Some(instant))
        } else {
            FlowInstruction::SendSignal(name, expr, Some(instant))
        }
    }
}
mk_new! { impl FlowInstruction =>
    Let: def_let (
        name: impl Into<Ident> = name.into(),
        expr: Expression = expr.into(),
    )
    InitEvent: init_event (
        name: impl Into<Ident> = name.into(),
    )
    UpdateEvent: update_event (
        name: impl Into<Ident> = name.into(),
        expr: Expression = expr.into(),
    )
    UpdateContext: update_ctx (
        name: impl Into<Ident> = name.into(),
        expr: Expression = expr.into(),
    )
    IfThrottle: if_throttle (
        flow_name: impl Into<Ident> = flow_name.into(),
        source_name: impl Into<Ident> = source_name.into(),
        delta: Constant = delta,
        instr: FlowInstruction = instr.into(),
    )
    IfChange: if_change (
        old_event_name: impl Into<Ident> = old_event_name.into(),
        signal: Expression = signal,
        then: FlowInstruction = then.into(),
    )
    IfActivated: if_activated (
        events: impl Into<Vec<Ident>> = events.into(),
        signals: impl Into<Vec<Ident>> = signals.into(),
        then: FlowInstruction = then.into(),
        els: Option<FlowInstruction> = els.map(Into::into),
    )
    ResetTimer: reset (
        name: impl Into<Ident> = name.into(),
        instant: impl Into<Ident> = instant.into(),
    )
    ComponentCall: comp_call (
        pat: Pattern = pat,
        mem_name: impl Into<Ident> = mem_name.into(),
        comp_name: impl Into<Ident> = comp_name.into(),
        inputs: impl Into<Vec<(Ident, Expression)>> = inputs.into(),
    )
    FunctionCall: fun_call (
        pat: Pattern = pat,
        name: impl Into<Ident> = name.into(),
        inputs: impl Into<Vec<Expression>> = inputs.into(),
    )
    HandleDelay: handle_delay(
        input_names: impl Iterator<Item = Ident> = input_names.collect(),
        arms: impl Iterator<Item = MatchArm> = arms.collect(),
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
    fn into_syn(self, stats: StatsMut) -> syn::Arm {
        let syn_pats = self.patterns.into_iter().map(|pat| pat.into_syn());
        let stmts = self.instr.into_syn2(stats);
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
        identifier: Ident,
    },
    /// An identifier call: `x`.
    Identifier {
        /// The identifier.
        identifier: Ident,
    },
    /// A call from the context: `ctx.s`.
    InContext {
        /// The flow called.
        flow: Ident,
    },
    /// A call from the context that will take the value: `ctx.s.take()`.
    TakeFromContext {
        /// The flow called.
        flow: Ident,
    },
    /// Some expression: `Some(v)`.
    Some {
        /// The value expression inside.
        expression: Box<Expression>,
    },
    /// None expression: `None`.
    None,
    /// Retrieve the instant of computation.
    Instant { ident: Ident },
}

mk_new! { impl Expression =>
    Literal: lit {
        literal: Constant = literal
    }
    Event: event {
        identifier: impl Into<Ident> = identifier.into()
    }
    Identifier: ident {
        identifier: impl Into<Ident> = identifier.into()
    }
    InContext: in_ctx {
        flow: impl Into<Ident> = flow.into()
    }
    TakeFromContext: take_from_ctx {
        flow: impl Into<Ident> = flow.into()
    }
    Some: some {
        expression: Expression = expression.into()
    }
    None: none {}
    Instant: instant { ident: Ident }
}

impl Expression {
    #[inline(always)]
    pub fn into_syn(self) -> syn::Expr {
        match self {
            Expression::Literal { literal } => literal.into_syn(),
            Expression::Event { identifier } => {
                let identifier = identifier.to_ref_var();
                parse_quote! { *#identifier }
            }
            Expression::Identifier { identifier } => {
                parse_quote! { #identifier }
            }
            Expression::InContext { flow } => {
                parse_quote! { self.context.#flow.get() }
            }
            Expression::TakeFromContext { flow } => {
                parse_quote! { std::mem::take(&mut self.context.#flow.0) }
            }
            Expression::Some { expression } => {
                let expression = expression.into_syn();
                parse_quote! { Some(#expression) }
            }
            Expression::None => parse_quote! { None },
            Expression::Instant { ident } => {
                // let test = std::time::Instant::now();
                // let truc = (test.duration_since(std::time::Instant::now()).as_millis()) as f64;
                let instant = ident.to_instant_var();
                parse_quote! { (#instant.duration_since(self.begin).as_millis()) as f64 }
            }
        }
    }
}
