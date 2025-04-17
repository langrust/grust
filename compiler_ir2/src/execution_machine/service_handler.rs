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
}

impl ToTokens for ServiceHandler {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let service_store_ident = &self.service_store_ident;
        let service_name = &self.service_struct_ident;

        let mut item_tokens = quote! {
            use futures::{stream::StreamExt, sink::SinkExt};
            use super::*;
        }
        .to_token_stream();
        self.flow_context.to_tokens(&mut item_tokens);

        // store all inputs in a `service_store`
        {
            // #TODO avoid allocations here
            let mut service_store_fields = vec![];
            let mut service_store_is_some_s = vec![];
            for FlowHandler { arriving_flow, .. } in self.flow_handlers.iter() {
                match arriving_flow {
                    ArrivingFlow::Channel(flow_name, flow_type, _) => {
                        let ident = flow_name;
                        service_store_fields
                            .push(quote! { #ident: Option<(#flow_type, std::time::Instant)> });
                        service_store_is_some_s.push(quote! { self.#ident.is_some() });
                    }
                    ArrivingFlow::Period(time_flow_name)
                    | ArrivingFlow::Deadline(time_flow_name) => {
                        let ident = time_flow_name;
                        service_store_fields
                            .push(quote! { #ident: Option<((), std::time::Instant)> });
                        service_store_is_some_s.push(quote! { self.#ident.is_some() });
                    }
                    ArrivingFlow::ServiceDelay(_) | ArrivingFlow::ServiceTimeout(_) => (),
                }
            }
            // service store
            quote! {
                #[derive(Default)]
                pub struct #service_store_ident {
                    #(#service_store_fields),*
                }
                impl #service_store_ident {
                    pub fn not_empty(&self) -> bool {
                        #(#service_store_is_some_s)||*
                    }
                }
            }
            .to_tokens(&mut item_tokens)
        }

        // create service structure
        {
            let mut service_fields = vec![
                quote! { begin: std::time::Instant },
                quote! { context: Context },
                quote! { delayed: bool },
                quote! { input_store: #service_store_ident },
            ];
            let mut field_values = vec![
                quote! { begin: std::time::Instant::now() },
                quote! { context },
                quote! { delayed },
                quote! { input_store },
            ];
            // with components states
            for ComponentInfo {
                state_ty_ident,
                field_ident,
                ..
            } in self.components_info.iter()
            {
                service_fields.push(quote! {
                    #field_ident: #state_ty_ident
                });
                field_values.push(field_ident.to_token_stream());
            }
            // and sending channels
            service_fields.push(quote! { output: futures::channel::mpsc::Sender<O> });
            service_fields
                .push(quote! { timer: futures::channel::mpsc::Sender<(T, std::time::Instant)> });
            field_values.push(quote! { output });
            field_values.push(quote! { timer });
            quote! {
                pub struct #service_name {
                    #(#service_fields),*
                }
            }
            .to_tokens(&mut item_tokens);

            // implement `init` and handler functions in an `impl`
            {
                // `init` function
                let mut impl_tokens = {
                    // create components states
                    let components_states = self.components_info.iter().map(
                        |ComponentInfo {
                             state_ty_ident,
                             field_ident,
                             ..
                         }| {
                            quote! {
                                let #field_ident =
                                    <#state_ty_ident as grust::core::Component>::init();
                            }
                        },
                    );
                    quote! {
                        pub fn init(
                            output: futures::channel::mpsc::Sender<O>,
                            timer: futures::channel::mpsc::Sender<(T, std::time::Instant)>,
                        ) -> #service_name {
                            let context = Context::init();
                            let delayed = true;
                            let input_store = Default::default();
                            #(#components_states)*
                            #service_name {
                                #(#field_values),*
                            }
                        }
                    }
                };

                for handler in self.flow_handlers.iter() {
                    handler.to_tokens(&mut impl_tokens)
                }

                // service handlers in an implementation block
                quote! {
                    #[inline]
                    pub async fn reset_time_constraints(
                        &mut self, instant: std::time::Instant
                    ) -> Result<(), futures::channel::mpsc::SendError> {
                        self.reset_service_delay(instant).await?;
                        self.delayed = false;
                        Ok(())
                    }
                }
                .to_tokens(&mut impl_tokens);
                quote! {
                    #[inline]
                    pub async fn send_output(
                        &mut self, output: O, instant: std::time::Instant
                    ) -> Result<(), futures::channel::mpsc::SendError> {
                        self.reset_service_timeout(instant).await?;
                        self.output.send(output).await?;
                        Ok(())
                    }
                }
                .to_tokens(&mut impl_tokens);
                quote! {
                    #[inline]
                    pub async fn send_timer(
                        &mut self, timer: T, instant: std::time::Instant
                    ) -> Result<(), futures::channel::mpsc::SendError> {
                        self.timer.send((timer, instant)).await?;
                        Ok(())
                    }
                }
                .to_tokens(&mut impl_tokens);

                // build `impl` block
                quote! { impl #service_name { #impl_tokens } }.to_tokens(&mut item_tokens);
            }
        }

        let mod_ident = &self.service_mod_ident;
        quote!(pub mod #mod_ident { #item_tokens }).to_tokens(tokens)
    }
}

#[derive(Debug, PartialEq)]
pub struct FlowHandler {
    pub arriving_flow: ArrivingFlow,
    pub instruction: FlowInstruction,
}

impl ToTokens for FlowHandler {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let instrs = &self.instruction;
        match &self.arriving_flow {
            ArrivingFlow::Channel(flow_name, flow_type, _) => {
                let instant = flow_name.to_instant_var();
                let function_name: Ident = flow_name.to_handle_fn();
                let ty = flow_type;
                let message = syn::LitStr::new(
                    format!("flow `{flow_name}` changes too frequently").as_str(),
                    Span::call_site(),
                );
                quote! {
                    pub async fn #function_name(
                        &mut self, #instant: std::time::Instant, #flow_name: #ty
                    ) -> Result<(), futures::channel::mpsc::SendError> {
                        if self.delayed {
                            // reset time constraints
                            self.reset_time_constraints(#instant).await?;
                            // reset all signals' update
                            self.context.reset();
                            // propagate changes
                            #instrs
                        } else {
                            // store in input_store
                            let unique =
                                self.input_store.#flow_name
                                    .replace((#flow_name, #instant));
                            assert!(unique.is_none(), #message);
                        }
                        Ok(())
                    }
                }
                .to_tokens(tokens)
            }
            ArrivingFlow::Period(time_flow_name) | ArrivingFlow::Deadline(time_flow_name) => {
                let instant = time_flow_name.to_instant_var();
                let function_name: Ident = time_flow_name.to_handle_fn();
                let message = syn::LitStr::new(
                    format!("flow `{time_flow_name}` changes too frequently").as_str(),
                    Span::call_site(),
                );
                quote! {
                    pub async fn #function_name(
                        &mut self,  #instant: std::time::Instant
                    ) -> Result<(), futures::channel::mpsc::SendError> {
                        if self.delayed {
                            // reset time constraints
                            self.reset_time_constraints(#instant).await?;
                            // reset all signals' update
                            self.context.reset();
                            // propagate changes
                            #instrs
                        } else {
                            // store in input_store
                            let unique =
                                self.input_store.#time_flow_name
                                    .replace(((), #instant));
                            assert!(unique.is_none(), #message);
                        }
                        Ok(())
                    }
                }
                .to_tokens(tokens)
            }
            ArrivingFlow::ServiceDelay(service_delay) => {
                let instant = Ident::instant_var();
                let function_name = service_delay.to_handle_fn();
                quote! {
                    pub async fn #function_name(
                        &mut self, #instant: std::time::Instant
                    ) -> Result<(), futures::channel::mpsc::SendError> {
                        // reset all signals' update
                        self.context.reset();
                        // propagate changes
                        #instrs
                        Ok(())
                    }
                }
                .to_tokens(tokens);
                let enum_ident = service_delay.to_ty();
                quote! {
                    #[inline]
                    pub async fn reset_service_delay(
                        &mut self, #instant: std::time::Instant
                    ) -> Result<(), futures::channel::mpsc::SendError> {
                        self.timer.send((T::#enum_ident, #instant)).await?;
                        Ok(())
                    }
                }
                .to_tokens(tokens)
            }
            ArrivingFlow::ServiceTimeout(service_timeout) => {
                let instant = service_timeout.to_instant_var();
                let function_name = service_timeout.to_handle_fn();
                quote! {
                    pub async fn #function_name(
                        &mut self, #instant: std::time::Instant
                    ) -> Result<(), futures::channel::mpsc::SendError> {
                        // reset time constraints
                        self.reset_time_constraints(#instant).await?;
                        // reset all signals' update
                        self.context.reset();
                        // propagate changes
                        #instrs
                        Ok(())
                    }
                }
                .to_tokens(tokens);
                let enum_ident = service_timeout.to_ty();
                quote! {
                    #[inline]
                    pub async fn reset_service_timeout(
                        &mut self, #instant: std::time::Instant
                    ) -> Result<(), futures::channel::mpsc::SendError> {
                        self.timer.send((T::#enum_ident, #instant)).await?;
                        Ok(())
                    }
                }
                .to_tokens(tokens)
            }
        }
    }
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
impl ToTokens for FlowInstruction {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            FlowInstruction::Let(ident, flow_expression) => {
                quote! { let #ident = #flow_expression; }.to_tokens(tokens)
            }
            FlowInstruction::InitEvent(ident) => {
                let ident = ident.to_ref_var();
                quote! { let #ident = &mut None; }.to_tokens(tokens)
            }
            FlowInstruction::UpdateEvent(ident, expr) => {
                let ident = ident.to_ref_var();
                quote! { *#ident = #expr; }.to_tokens(tokens)
            }
            FlowInstruction::UpdateContext(ident, flow_expression) => {
                quote! { self.context.#ident.set(#flow_expression); }.to_tokens(tokens)
            }
            FlowInstruction::SendSignal(name, send_expr, instant) => {
                let enum_ident = name.to_camel();
                let instant = if let Some(instant) = instant {
                    instant.to_instant_var()
                } else {
                    Ident::instant_var()
                };
                quote! {
                    self.send_output(O::#enum_ident(#send_expr, #instant), #instant).await?;
                }
                .to_tokens(tokens)
            }
            FlowInstruction::SendEvent(name, event_expr, send_expr, instant) => {
                let enum_ident = name.to_camel();
                let instant = if let Some(instant) = instant {
                    instant.to_instant_var()
                } else {
                    Ident::instant_var()
                };
                quote! {
                    if let Some(#name) = #event_expr {
                        self.send_output(O::#enum_ident(#send_expr, #instant), #instant).await?;
                    }
                }
                .to_tokens(tokens)
            }
            FlowInstruction::IfThrottle(receiver_name, source_name, delta, instruction) => quote! {
                if (self.context.#receiver_name.get() - #source_name).abs() >= #delta {
                    #instruction
                }
            }
            .to_tokens(tokens),
            FlowInstruction::IfChange(old_event_name, signal, then) => {
                let old_event_ident = old_event_name;
                let expr = signal;
                quote! {
                    if self.context.#old_event_ident.get() != #expr {
                        #then
                    }
                }
                .to_tokens(tokens)
            }
            FlowInstruction::ResetTimer(timer_name, import_name) => {
                let enum_ident = timer_name.to_ty();
                let instant = import_name.to_instant_var();
                quote! { self.send_timer(T::#enum_ident, #instant).await?; }.to_tokens(tokens)
            }
            FlowInstruction::ComponentCall(pattern, memory_name, comp_name, inputs_fields) => {
                let outputs = pattern;
                let mem_ident = memory_name.to_field();
                let input_ty = comp_name.to_input_ty();
                let state_ty = comp_name.to_state_ty();

                let input_fields = inputs_fields.iter().map(|(field_name, input)| {
                    quote! { #field_name : #input }
                });

                quote! {
                    let #outputs = <#state_ty as grust::core::Component>::step(
                        &mut self.#mem_ident,
                        #input_ty {
                            #(#input_fields),*
                        }
                    );
                }
                .to_tokens(tokens)
            }
            FlowInstruction::FunctionCall(pattern, function_name, inputs) => {
                let outputs = &pattern;
                let function_ident = function_name.to_field();
                quote! {
                    let #outputs = self.#function_ident(#(#inputs),*);
                }
                .to_tokens(tokens)
            }
            FlowInstruction::HandleDelay(input_flows, match_arms) => {
                let input_flows = input_flows.iter().map(|name| {
                    quote! { self.input_store.#name.take() }
                });
                let instant = Ident::instant_var();
                quote! {
                    if self.input_store.not_empty() {
                        self.reset_time_constraints(#instant).await?;
                        match (#(#input_flows),*) {
                            #(#match_arms)*
                        }
                    } else {
                        self.delayed = true;
                    }
                }
                .to_tokens(tokens)
            }
            FlowInstruction::IfActivated(events, signals, then, els) => {
                let activation_cond = events
                    .iter()
                    .map(|ident| {
                        let ident = ident.to_ref_var();
                        quote! { #ident.is_some() }
                    })
                    .chain(signals.iter().map(|ident| {
                        quote! { self.context.#ident.is_new() }
                    }));

                if events.is_empty() && signals.is_empty() {
                    if let Some(els) = els {
                        els.to_tokens(tokens)
                    } else {
                        ()
                    }
                } else {
                    let els = els.as_ref().map(|els| quote!(else { #els }));
                    quote! {
                        if #(#activation_cond)||* {
                            #then
                        } #els
                    }
                    .to_tokens(tokens);
                }
            }
            FlowInstruction::Seq(instrs) => {
                let mut stack = vec![instrs.iter()];
                while let Some(mut iter) = stack.pop() {
                    if let Some(instr) = iter.next() {
                        stack.push(iter);
                        if let Self::Seq(subs) = instr {
                            stack.push(subs.iter());
                            continue;
                        } else {
                            instr.to_tokens(tokens)
                        }
                    }
                }
            }
            FlowInstruction::Para(method_map) => {
                let para_futures = method_map.iter().flat_map(|(_method, para_instrs)| {
                    para_instrs.iter().map(|instr| quote! { async { #instr } })
                });
                quote! {
                    tokio::join!(#(#para_futures),*);
                }
                .to_tokens(tokens)
            }
        }
    }
}

impl FlowInstruction {
    pub fn seq(mut vec: Vec<Self>) -> Self {
        if vec.len() == 1 {
            vec.pop().expect("len is `1`")
        } else {
            Self::Seq(vec)
        }
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

impl ToTokens for MatchArm {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let pats = &self.patterns;
        let instr = &self.instr;
        quote! {
            (#(#pats),*) => { #instr }
        }
        .to_tokens(tokens)
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
impl ToTokens for Expression {
    #[inline(always)]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Expression::Literal { literal } => literal.to_tokens(tokens),
            Expression::Event { identifier } => {
                let identifier = identifier.to_ref_var();
                quote! { *#identifier }.to_tokens(tokens)
            }
            Expression::Identifier { identifier } => identifier.to_tokens(tokens),
            Expression::InContext { flow } => quote! { self.context.#flow.get() }.to_tokens(tokens),
            Expression::TakeFromContext { flow } => {
                quote! { self.context.#flow.take() }.to_tokens(tokens)
            }
            Expression::Some { expression } => quote! { Some(#expression) }.to_tokens(tokens),
            Expression::None => quote! { None }.to_tokens(tokens),
            Expression::Instant { ident } => {
                let instant = ident.to_instant_var();
                quote! { (#instant.duration_since(self.begin).as_millis()) as f64 }
                    .to_tokens(tokens)
            }
        }
    }
}
