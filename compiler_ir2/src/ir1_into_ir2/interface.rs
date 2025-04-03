prelude! {
    ir1::interface::{FlowExport, FlowImport, Interface, Service},
    execution_machine::{
        ServiceHandler, InputHandler, RuntimeLoop, ExecutionMachine,TimingEvent, InterfaceFlow,
    },
}

impl Ir1IntoIr2<&mut Ctx> for Interface {
    type Ir2 = ExecutionMachine;
    fn into_ir2(mut self, symbol_table: &mut Ctx) -> ExecutionMachine {
        if self.services.is_empty() {
            return Default::default();
        }

        // used to store timers from every `sample`, `scan`, `timeout` operators (etc)
        let mut timing_events = vec![];

        // get functions to propagate input flows inside every service
        let services_handlers: Vec<ServiceHandler> = self
            .services
            .into_iter()
            .map(|service| {
                service.into_ir2(ir1::ctx::Full::new(
                    &mut self.imports,
                    &self.exports,
                    &mut timing_events,
                    symbol_table,
                ))
            })
            .collect();

        // get functions to call the right services for every input arrival
        let mut input_handlers = HashMap::new();
        services_handlers.iter().for_each(|service_handler| {
            service_handler
                .flow_handlers
                .iter()
                .for_each(|flow_handler| {
                    input_handlers
                        .entry(&flow_handler.arriving_flow)
                        .or_insert_with(|| vec![])
                        .push(service_handler.service_ident.clone())
                })
        });
        // put the latest in a runtime loop
        let runtime_loop = RuntimeLoop {
            input_handlers: input_handlers
                .into_iter()
                .map(|(ref_to, services)| InputHandler {
                    arriving_flow: ref_to.clone(),
                    services,
                })
                .collect(),
        };

        // get input and output flows
        let input_flows = self
            .imports
            .into_values()
            .filter_map(|import| import.into_ir2(symbol_table))
            .collect();
        let output_flows = self
            .exports
            .into_values()
            .map(|export| export.into_ir2(symbol_table))
            .collect();

        // construct execution machine
        ExecutionMachine {
            runtime_loop,
            services_handlers,
            input_flows,
            output_flows,
            timing_events,
        }
    }
}

impl Ir1IntoIr2<&'_ Ctx> for FlowImport {
    type Ir2 = Option<InterfaceFlow>;

    fn into_ir2(self, symbol_table: &Ctx) -> Self::Ir2 {
        if self.flow_type.eq(&Typ::event(Typ::unit())) {
            None
        } else {
            Some(InterfaceFlow {
                path: self.path,
                identifier: symbol_table.get_name(self.id).clone(),
                typ: self.flow_type,
            })
        }
    }
}

impl Ir1IntoIr2<&'_ Ctx> for FlowExport {
    type Ir2 = InterfaceFlow;

    fn into_ir2(self, symbol_table: &Ctx) -> Self::Ir2 {
        InterfaceFlow {
            path: self.path,
            identifier: symbol_table.get_name(self.id).clone(),
            typ: self.flow_type,
        }
    }
}

impl Ir1IntoIr2<ir1::ctx::Full<'_, TimingEvent>> for Service {
    type Ir2 = ServiceHandler;
    fn into_ir2(mut self, mut ctx: ir1::ctx::Full<TimingEvent>) -> ServiceHandler {
        let flows_context = self.get_flows_context(&ctx, ctx.exports.values());
        ctx.local();
        let builder: flow_instr::Builder<'_> = flow_instr::Builder::new(
            &mut self,
            ctx.ctx0,
            flows_context,
            ctx.imports,
            ctx.exports,
            ctx.timings,
        );
        let service_handler = service_handler::build(builder);
        ctx.global();
        service_handler
    }
}

mod service_handler {
    prelude! {
        {
            execution_machine::{
                FlowHandler, FlowInstruction, MatchArm, ServiceHandler, ArrivingFlow,
            },
            Pattern,
        },
        synced::generic::{Builder, Synced},
    }
    use super::{clean_synced, flow_instr, from_synced};

    /// Compute the instruction propagating the changes of the input flow.
    fn propagate<'a>(
        ctx: &mut flow_instr::Builder<'a>,
        stmt_id: usize,
        flow_id: usize,
    ) -> FlowInstruction {
        if ctx.is_delay(flow_id) {
            propagate_input_store(ctx, flow_id)
        } else {
            ctx.set_multiple_inputs(false);
            flow_instruction(ctx, std::iter::once(stmt_id))
        }
    }

    /// Compute the instruction propagating the changes of the input store.
    fn propagate_input_store<'a>(
        ctx: &mut flow_instr::Builder<'a>,
        delay_id: usize,
    ) -> FlowInstruction {
        debug_assert!(ctx.is_clear());
        debug_assert!(ctx.is_delay(delay_id));
        let ctx0 = ctx.ctx0();

        // this is an ORDERED list of the input flows
        let inputs = ctx.inputs().collect::<Vec<_>>();
        let flows_names = inputs
            .iter()
            .map(|(_, import_id)| ctx0.get_name(*import_id).clone());

        // Create the handler of the delay timer.
        // It propagates all changes stored in the service_store by matching
        // each one of its elements (that are of type Option<(Value, Instant)>).
        let rng = 0..(2i64.pow(inputs.len() as u32));
        let arms = rng.map(|mut i| {
            // gather the flows that have been modified
            let imports = inputs
                .iter()
                .filter_map(|(stmt_id, _)| {
                    let res = if i & 1 == 1 { Some(*stmt_id) } else { None };
                    i = i >> 1;
                    res
                })
                .collect::<Vec<_>>();
            let patterns = inputs
                .iter()
                .map(|(stmt_id, import_id)| {
                    if imports.contains(stmt_id) {
                        let flow_name = ctx0.get_name(*import_id);
                        let instant = flow_name.to_instant_var();
                        if ctx0.is_timer(*import_id) {
                            Pattern::some(Pattern::tuple(vec![
                                Pattern::literal(Constant::unit(Default::default())),
                                Pattern::ident(instant),
                            ]))
                        } else {
                            Pattern::some(Pattern::tuple(vec![
                                Pattern::ident(flow_name.clone()),
                                Pattern::ident(instant),
                            ]))
                        }
                    } else {
                        Pattern::none()
                    }
                })
                .collect();
            // compute the instruction that will propagate changes
            ctx.set_multiple_inputs(true);
            let instr = flow_instruction(ctx, imports.into_iter());
            MatchArm::new(patterns, instr)
        });

        FlowInstruction::handle_delay(flows_names, arms)
    }

    /// Compute the instruction propagating the changes of the input flows.
    fn flow_instruction<'a>(
        ctx: &mut flow_instr::Builder<'a>,
        imports: impl Iterator<Item = usize>,
    ) -> FlowInstruction {
        debug_assert!(ctx.is_clear());
        // construct subgraph representing the propagation of 'imports'
        let subgraph = &ctx.graph().subgraph(imports);

        let synced = if ctx.conf.para {
            // if config is 'para' then build 'synced' with //-algo
            if subgraph.node_count() == 0 {
                return FlowInstruction::seq(vec![]);
            }
            let builder = Builder::<flow_instr::Builder>::new(subgraph);
            builder.run(ctx).expect("oh no")
        } else {
            // else, construct an ordered sequence of instructions
            let ord_instrs = graph::toposort(subgraph, None).expect("no cycle expected");
            let seq: Vec<_> = ord_instrs
                .into_iter()
                .map(|i| Synced::instr(i, ctx))
                .collect();
            if seq.is_empty() {
                return FlowInstruction::seq(vec![]);
            }
            Synced::seq(seq, ctx)
        };

        // puts the exports out of parallel instructions
        let synced = clean_synced::run(ctx, synced);
        // produce the corresponding [ir2] instruction
        let instr = from_synced::run(ctx, synced);

        ctx.clear();
        instr
    }

    /// Compute the input flow's handler.
    fn flow_handler<'a>(
        ctx: &mut flow_instr::Builder<'a>,
        stmt_id: usize,
        flow_id: usize,
    ) -> FlowHandler {
        // construct the instruction to perform
        let instruction = propagate(ctx, stmt_id, flow_id);

        let flow_name = ctx.get_name(flow_id).clone();
        // determine whether this arriving flow is a timing event
        let arriving_flow = if ctx.is_delay(flow_id) {
            ArrivingFlow::ServiceDelay(flow_name)
        } else if ctx.is_period(flow_id) {
            ArrivingFlow::Period(flow_name)
        } else if ctx.is_deadline(flow_id) {
            ArrivingFlow::Deadline(flow_name)
        } else if ctx.is_timeout(flow_id) {
            ArrivingFlow::ServiceTimeout(flow_name)
        } else {
            let flow_type = ctx.get_typ(flow_id).clone();
            let path = ctx.get_path(flow_id).clone();
            ArrivingFlow::Channel(flow_name, flow_type, path)
        };

        FlowHandler {
            arriving_flow,
            instruction,
        }
    }

    /// Compute the service handler.
    pub fn build<'a>(mut ctx: flow_instr::Builder<'a>) -> ServiceHandler {
        // get service's name
        let service = ctx.service_name().clone();
        // create flow handlers according to propagations of every incoming flows
        let flow_handlers: Vec<_> = ctx
            .service_imports()
            .map(|(stmt_id, import_id)| flow_handler(&mut ctx, stmt_id, import_id))
            .collect();
        // destroy 'ctx'
        let (flows_context, components) = ctx.destroy();

        ServiceHandler::new(service, components, flow_handlers, flows_context)
    }
}

mod flow_instr {
    prelude! {
        quote::format_ident,
        graph::{DfsEvent::*, DiGraphMap},
        ir1::{
            flow,
            interface::{
                FlowDeclaration, FlowInstantiation,
                FlowStatement, FlowImport, FlowExport,
            },
            IdentifierCreator, Service,
        },
        ir1_into_ir2::trigger,
        execution_machine::{
            Expression, FlowInstruction, TimingEvent, TimingEventKind,
        },
    }

    // use ir1_into_ir2::trigger::{self, TriggersGraph};

    /// A context to build [FlowInstruction]s.
    pub struct Builder<'a> {
        /// Context of the service.
        flows_context: ir1::ctx::Flows,
        /// Symbol table.
        ctx0: &'a Ctx,
        /// Events currently triggered during a traversal.
        events: HashSet<usize>,
        /// Signals currently defined during a traversal.
        signals: HashSet<usize>,
        /// Maps on_change event indices to the indices of signals containing their previous values.
        on_change_events: HashMap<usize, usize>,
        /// Maps statement indices to the indices and kinds of their timing_events.
        stmts_timers: HashMap<usize, usize>,
        /// Tells if we handle multiple incoming flows.
        multiple_inputs: bool,
        /// Maps statement to their related imports.
        stmts_imports: HashMap<usize, Vec<(usize, usize)>>,
        /// Service to build propagations for.
        service: &'a Service,
        /// Map from id to import.
        imports: &'a HashMap<usize, FlowImport>,
        /// Map from id to export.
        exports: &'a HashMap<usize, FlowExport>,
        /// Called components.
        components: Vec<Ident>,
        /// Triggers graph,
        graph: trigger::Graph<'a>,
    }
    impl ops::Deref for Builder<'_> {
        type Target = Ctx;
        fn deref(&self) -> &Self::Target {
            self.ctx0
        }
    }

    impl<'a> Builder<'a> {
        /// Create a Builder.
        ///
        /// After creating the builder, you only need to [super::service_handler::build].
        pub fn new(
            service: &'a mut Service,
            ctx0: &'a mut Ctx,
            mut flows_context: ir1::ctx::Flows,
            imports: &'a mut HashMap<usize, FlowImport>,
            exports: &'a HashMap<usize, FlowExport>,
            timing_events: &'a mut Vec<TimingEvent>,
        ) -> Self {
            let mut identifier_creator = IdentifierCreator::from(
                service.get_flows_names(ctx0).chain(
                    imports
                        .values()
                        .map(|import| ctx0.get_name(import.id).clone()),
                ),
            );
            let mut components = vec![];
            // retrieve timer and onchange events from service
            let (stmts_timers, on_change_events) = Self::build_stmt_events(
                &mut identifier_creator,
                service,
                ctx0,
                &mut flows_context,
                imports,
                timing_events,
                &mut components,
            );
            // add events related to service's constraints
            Self::build_constraint_events(
                &mut identifier_creator,
                service,
                ctx0,
                imports,
                timing_events,
            );
            // add edge in graph between any import (excluding service delay) and `time` stmts
            service
                .statements
                .iter()
                .filter(|(_, stmt)| match stmt {
                    FlowStatement::Declaration(FlowDeclaration { expr, .. })
                    | FlowStatement::Instantiation(FlowInstantiation { expr, .. }) => {
                        match expr.kind {
                            flow::Kind::Time { .. } => true,
                            _ => false,
                        }
                    }
                })
                .for_each(|(stmt_id, _)| {
                    for import_stmt_id in imports.keys() {
                        if service.graph.edges(*import_stmt_id).next().is_some() {
                            service.graph.add_edge(*import_stmt_id, *stmt_id, ());
                        }
                    }
                });

            // create triggered graph
            let graph = trigger::Graph::new(ctx0, service, imports);
            // construct [stmt -> imports]
            let stmts_imports = Self::build_stmts_imports(&graph.graph(), imports);

            Builder {
                flows_context,
                ctx0,
                on_change_events,
                stmts_timers,
                events: HashSet::new(),
                signals: HashSet::new(),
                multiple_inputs: false,
                stmts_imports,
                service,
                imports,
                exports,
                components,
                graph,
            }
        }

        pub fn get_stmt(&self, stmt_id: usize) -> Option<&FlowStatement> {
            self.service.statements.get(&stmt_id)
        }
        pub fn get_import(&self, import_id: usize) -> Option<&FlowImport> {
            self.imports.get(&import_id)
        }
        pub fn get_export(&self, export_id: usize) -> Option<&FlowExport> {
            self.exports.get(&export_id)
        }
        pub fn ctx0(&self) -> &'a Ctx {
            self.ctx0
        }
        pub fn graph(&self) -> &trigger::Graph<'a> {
            &self.graph
        }
        pub fn service_imports(&self) -> impl Iterator<Item = (usize, usize)> + 'a {
            self.imports
                .iter()
                .filter(|(stmt_id, import)| {
                    // 'service_delay' is not in the graph
                    // (it does not trigger instructions but the propagation of the 'input_store')
                    self.ctx0.is_service_delay(self.service.id, import.id)
                        || self.service.graph.edges(**stmt_id).next().is_some()
                })
                .map(|(stmt_id, import)| (*stmt_id, import.id))
        }
        pub fn service_name(&self) -> &Ident {
            self.ctx0.get_name(self.service.id)
        }
        pub fn inputs(&self) -> impl Iterator<Item = (usize, usize)> + 'a {
            self.service_imports().filter(|(_, import_id)| {
                !(self.ctx0.is_delay(*import_id) || self.ctx0.is_timeout(*import_id))
            })
        }
        pub fn set_multiple_inputs(&mut self, multiple_inputs: bool) {
            self.multiple_inputs = multiple_inputs
        }
        pub fn destroy(self) -> (ir1::ctx::Flows, Vec<Ident>) {
            (self.flows_context, self.components)
        }

        /// Clear the builder: events and signals sets
        pub fn clear(&mut self) {
            self.events.clear();
            self.signals.clear();
        }

        /// Tells if the builder is cleared.
        pub fn is_clear(&self) -> bool {
            self.events.is_empty() && self.signals.is_empty()
        }

        /// Constructs the map from statement to related imports.
        fn build_stmts_imports(
            graph: &DiGraphMap<usize, ()>,
            imports: &HashMap<usize, FlowImport>,
        ) -> HashMap<usize, Vec<(usize, usize)>> {
            let mut stmts_imports = HashMap::new();
            for (import_stmt_id, import) in imports.iter() {
                graph::visit::depth_first_search(
                    graph,
                    std::iter::once(*import_stmt_id),
                    |event| match event {
                        CrossForwardEdge(_, child) | BackEdge(_, child) | TreeEdge(_, child) => {
                            stmts_imports
                                .entry(child)
                                .or_insert(vec![])
                                .push((*import_stmt_id, import.id))
                        }
                        Discover(_, _) | Finish(_, _) => {}
                    },
                );
            }
            stmts_imports
        }

        /// Adds events related to statements.
        fn build_stmt_events(
            identifier_creator: &mut IdentifierCreator,
            service: &mut Service,
            symbols: &mut Ctx,
            flows_context: &mut ir1::ctx::Flows,
            imports: &mut HashMap<usize, FlowImport>,
            timing_events: &mut Vec<TimingEvent>,
            components: &mut Vec<Ident>,
        ) -> (HashMap<usize, usize>, HashMap<usize, usize>) {
            // collects components, timing events, on_change_events that are present in the service
            let mut stmts_timers = HashMap::new();
            let mut on_change_events = HashMap::new();
            service.statements.iter().for_each(|(stmt_id, statement)| {
                let stmt_id = *stmt_id;
                match statement {
                    FlowStatement::Declaration(FlowDeclaration { pattern, expr, .. })
                    | FlowStatement::Instantiation(FlowInstantiation { pattern, expr, .. }) => {
                        match &expr.kind {
                            flow::Kind::Ident { .. }
                            | flow::Kind::Throttle { .. }
                            | flow::Kind::Merge { .. }
                            | flow::Kind::Time { .. }
                            | flow::Kind::Persist { .. }
                            | flow::Kind::FunctionCall { .. } => (),
                            flow::Kind::OnChange { .. } => {
                                // get the identifier of the created event
                                let mut ids = pattern.identifiers();
                                debug_assert!(ids.len() == 1);
                                let flow_event_id = ids.pop().unwrap();
                                let event_name = symbols.get_name(flow_event_id).clone();

                                // add new event into the identifier creator
                                let fresh_name = identifier_creator.new_identifier_with(
                                    event_name.loc(),
                                    "",
                                    &event_name.to_string(),
                                    "old",
                                );
                                let typing = symbols.get_typ(flow_event_id).clone();
                                let kind = symbols.get_flow_kind(flow_event_id).clone();
                                let fresh_id = symbols.insert_fresh_flow(
                                    fresh_name.clone(),
                                    kind,
                                    typing.clone(),
                                );

                                // add event_old in flows_context
                                flows_context.add_element(fresh_name, &typing);

                                // push in on_change_events
                                on_change_events.insert(flow_event_id, fresh_id);
                            }
                            flow::Kind::Sample { period_ms, .. }
                            | flow::Kind::Scan { period_ms, .. } => {
                                // add new timing event into the identifier creator
                                let flow_name =
                                    symbols.get_name(pattern.identifiers().pop().unwrap());
                                let fresh_name = identifier_creator.fresh_identifier(
                                    flow_name.loc(),
                                    "period",
                                    flow_name.to_string(),
                                );
                                let typing = Typ::event(Typ::unit());
                                let fresh_id =
                                    symbols.insert_fresh_period(fresh_name.clone(), *period_ms);

                                // add timing_event in imports
                                let fresh_statement_id = symbols.get_fresh_id();
                                imports.insert(
                                    fresh_statement_id,
                                    FlowImport {
                                        import_token: Default::default(),
                                        id: fresh_id,
                                        path: format_ident!("{fresh_name}").into(),
                                        colon_token: Default::default(),
                                        flow_type: typing,
                                        semi_token: Default::default(),
                                    },
                                );
                                // add timing_event in graph
                                service.graph.add_node(fresh_statement_id);
                                service.graph.add_edge(fresh_statement_id, stmt_id, ());

                                // push timing_event
                                stmts_timers.insert(stmt_id, fresh_id);
                                timing_events.push(TimingEvent {
                                    identifier: fresh_name,
                                    kind: TimingEventKind::Period(period_ms.clone()),
                                });
                            }
                            flow::Kind::Timeout { deadline, .. } => {
                                // add new timing event into the identifier creator
                                let flow_name =
                                    symbols.get_name(pattern.identifiers().pop().unwrap());
                                let fresh_name = identifier_creator.fresh_identifier(
                                    flow_name.loc(),
                                    "timeout",
                                    &flow_name.to_string(),
                                );
                                let typing = Typ::event(Typ::unit());
                                let fresh_id =
                                    symbols.insert_fresh_deadline(fresh_name.clone(), *deadline);

                                // add timing_event in imports
                                let fresh_statement_id = symbols.get_fresh_id();
                                imports.insert(
                                    fresh_statement_id,
                                    FlowImport {
                                        import_token: Default::default(),
                                        id: fresh_id,
                                        path: format_ident!("{fresh_name}").into(),
                                        colon_token: Default::default(),
                                        flow_type: typing,
                                        semi_token: Default::default(),
                                    },
                                );

                                // add timing_event in graph
                                service.graph.add_node(fresh_statement_id);
                                service.graph.add_edge(fresh_statement_id, stmt_id, ());

                                // push timing_event
                                stmts_timers.insert(stmt_id, fresh_id);
                                timing_events.push(TimingEvent {
                                    identifier: fresh_name,
                                    kind: TimingEventKind::Timeout(deadline.clone()),
                                })
                            }
                            flow::Kind::ComponentCall { component_id, .. } => {
                                components.push(symbols.get_name(*component_id).clone())
                            }
                        }
                    }
                };
            });

            (stmts_timers, on_change_events)
        }

        /// Adds events related to service's constraints.
        fn build_constraint_events(
            identifier_creator: &mut IdentifierCreator,
            service: &mut Service,
            symbols: &mut Ctx,
            imports: &mut HashMap<usize, FlowImport>,
            timing_events: &mut Vec<TimingEvent>,
        ) {
            // add service delay
            let min_delay = service.time_range.0;
            // add new timing event into the identifier creator
            let fresh_name = {
                let s = symbols.get_name(service.id);
                identifier_creator.fresh_identifier(s.loc(), "delay", &s.to_string())
            };
            let typing = Typ::event(Typ::unit());
            let fresh_id = symbols.insert_service_delay(fresh_name.clone(), service.id, min_delay);
            // add timing_event in imports
            let fresh_statement_id = symbols.get_fresh_id();
            imports.insert(
                fresh_statement_id,
                FlowImport {
                    import_token: Default::default(),
                    id: fresh_id,
                    path: format_ident!("{fresh_name}").into(),
                    colon_token: Default::default(),
                    flow_type: typing,
                    semi_token: Default::default(),
                },
            );
            // add timing_event in graph
            service.graph.add_node(fresh_statement_id);
            // push timing_event
            timing_events.push(TimingEvent {
                identifier: fresh_name,
                kind: TimingEventKind::ServiceDelay(min_delay),
            });

            // add service timeout
            let max_timeout = service.time_range.1;
            // add new timing event into the identifier creator
            let fresh_name = {
                let s = symbols.get_name(service.id);
                identifier_creator.fresh_identifier(s.loc(), "timeout", &s.to_string())
            };
            let typing = Typ::event(Typ::unit());
            let fresh_id =
                symbols.insert_service_timeout(fresh_name.clone(), service.id, max_timeout);
            // add timing_event in imports
            let fresh_statement_id = symbols.get_fresh_id();
            imports.insert(
                fresh_statement_id,
                FlowImport {
                    import_token: Default::default(),
                    id: fresh_id,
                    path: format_ident!("{fresh_name}").into(),
                    colon_token: Default::default(),
                    flow_type: typing,
                    semi_token: Default::default(),
                },
            );
            // add timing_event in graph
            service.graph.add_node(fresh_statement_id);
            service.statements.iter().for_each(|(stmt_id, stmt)| {
                if stmt.is_comp_call() {
                    // todo: why particularly components?
                    service.graph.add_edge(fresh_statement_id, *stmt_id, ());
                }
            });
            // push timing_event
            timing_events.push(TimingEvent {
                identifier: fresh_name,
                kind: TimingEventKind::ServiceTimeout(max_timeout),
            });
        }

        /// Returns the import flow that have triggered the stmt.
        fn get_stmt_import(&self, stmt_id: usize) -> usize {
            let imports = self
                .stmts_imports
                .get(&stmt_id)
                .expect("there should be imports");
            debug_assert!(!imports.is_empty());

            imports
                .iter()
                .filter(|(_, import_id)| {
                    self.events.contains(import_id) || self.signals.contains(import_id)
                })
                .next()
                .unwrap()
                .1
        }

        /// Compute the instruction that will init the events.
        pub fn init_events<'b>(&'b self) -> impl Iterator<Item = FlowInstruction> + 'b {
            self.events
                .iter()
                .filter(|event_id| !self.is_timer(**event_id))
                .map(|event_id| FlowInstruction::init_event(self.get_name(*event_id).clone()))
        }

        /// Compute the instruction from an import.
        pub fn handle_import(&mut self, flow_id: usize) -> FlowInstruction {
            if self.get_flow_kind(flow_id).is_event() {
                // add to events set
                self.events.insert(flow_id);
                if !self.is_timer(flow_id) {
                    // store the event in the local reference
                    let event_name = self.get_name(flow_id);
                    let expr = Expression::some(Expression::ident(event_name.clone()));
                    self.define_event(flow_id, expr)
                } else if self.is_period(flow_id) {
                    // reset periodic timer
                    self.reset_timer(flow_id, flow_id)
                } else {
                    // if timer other than period, then do nothing
                    FlowInstruction::seq(vec![])
                }
            } else {
                // add to signals set
                self.signals.insert(flow_id);
                if let Some(update) = self.update_ctx(flow_id) {
                    // update the context if necessary
                    update
                } else {
                    FlowInstruction::seq(vec![])
                }
            }
        }

        /// Compute the instruction from an expression flow.
        pub fn handle_expr(
            &mut self,
            stmt_id: usize,
            pattern: &ir1::stmt::Pattern,
            expr: &flow::Expr,
        ) -> FlowInstruction {
            let dependencies = expr.get_dependencies();
            match &expr.kind {
                flow::Kind::Ident { id } => self.handle_ident(pattern, *id),
                flow::Kind::Sample { .. } => self.handle_sample(stmt_id, pattern, dependencies),
                flow::Kind::Scan { .. } => self.handle_scan(stmt_id, pattern, dependencies),
                flow::Kind::Timeout { .. } => self.handle_timeout(stmt_id, pattern, dependencies),
                flow::Kind::Throttle { delta, .. } => {
                    self.handle_throttle(pattern, dependencies, delta.clone())
                }
                flow::Kind::OnChange { .. } => self.handle_on_change(pattern, dependencies),
                flow::Kind::Persist { .. } => self.handle_persist(pattern, dependencies),
                flow::Kind::Merge { .. } => self.handle_merge(pattern, dependencies),
                flow::Kind::Time { loc } => self.handle_time(stmt_id, pattern, *loc),
                flow::Kind::ComponentCall {
                    component_id,
                    inputs,
                } => self.handle_component_call(pattern, *component_id, inputs),
                flow::Kind::FunctionCall {
                    function_id,
                    inputs,
                } => self.handle_function_call(pattern, *function_id, inputs),
            }
        }

        /// Compute the instruction from an identifier expression.
        fn handle_ident(
            &mut self,
            pattern: &ir1::stmt::Pattern,
            id_source: usize,
        ) -> FlowInstruction {
            // get the id of pattern's flow, debug-check there is only one flow
            let mut ids = pattern.identifiers();
            debug_assert!(ids.len() == 1);
            let id_pattern = ids.pop().unwrap();

            // insert instruction only if source is a signal or an activated event
            let def = if self.get_flow_kind(id_source).is_signal() {
                let expr = self.get_signal(id_source);
                self.define_signal(id_pattern, expr)
            } else {
                let expr = self.get_event(id_source);
                self.define_event(id_pattern, expr)
            };

            if let Some(update) = self.update_ctx(id_pattern) {
                FlowInstruction::seq(vec![def, update])
            } else {
                def
            }
        }

        /// Compute the instruction from a sample expression.
        fn handle_sample(
            &mut self,
            stmt_id: usize,
            pattern: &ir1::stmt::Pattern,
            mut dependencies: Vec<usize>,
        ) -> FlowInstruction {
            // get the id of pattern's flow, debug-check there is only one flow
            let mut ids = pattern.identifiers();
            debug_assert!(ids.len() == 1);
            let id_pattern = ids.pop().unwrap();
            let flow_name = self.get_name(id_pattern);

            // get the source id, debug-check there is only one flow
            debug_assert!(dependencies.len() == 1);
            let id_source = dependencies.pop().unwrap();
            let source_name = self.get_name(id_source);

            let timer_id = self.stmts_timers[&stmt_id];

            let mut instrs = vec![];
            // source is an event, look if it is defined
            if self.events.contains(&id_source) {
                // if activated, store event value in context
                let update = FlowInstruction::update_ctx(
                    source_name.clone(),
                    Expression::event(source_name.clone()),
                );
                instrs.push(FlowInstruction::if_activated(
                    vec![source_name.clone()],
                    [],
                    update,
                    None,
                ))
            }
            // if timing event is activated
            if self.events.contains(&timer_id) {
                // update signal by taking from stored event value
                let take_update = FlowInstruction::update_ctx(
                    flow_name.clone(),
                    Expression::take_from_ctx(source_name.clone()),
                );
                instrs.push(take_update)
            }

            FlowInstruction::seq(instrs)
        }

        /// Compute the instruction from a scan expression.
        fn handle_scan(
            &mut self,
            stmt_id: usize,
            pattern: &ir1::stmt::Pattern,
            mut dependencies: Vec<usize>,
        ) -> FlowInstruction {
            // get the id of pattern's flow, debug-check there is only one flow
            let mut ids = pattern.identifiers();
            debug_assert!(ids.len() == 1);
            let id_pattern = ids.pop().unwrap();

            // get the source id, debug-check there is only one flow
            debug_assert!(dependencies.len() == 1);
            let id_source = dependencies.pop().unwrap();

            let timer_id = self.stmts_timers[&stmt_id];

            // timer is an event, look if it is defined
            if self.events.contains(&timer_id) {
                // if activated, create event
                let expr = Expression::some(self.get_signal(id_source));
                self.define_event(id_pattern, expr)
            } else {
                // 'scan' can be activated by the source signal, but it won't do anything
                FlowInstruction::seq(vec![])
            }
        }

        /// Compute the instruction from a timeout expression.
        fn handle_timeout(
            &mut self,
            stmt_id: usize,
            pattern: &ir1::stmt::Pattern,
            mut dependencies: Vec<usize>,
        ) -> FlowInstruction {
            // get the id of pattern's flow, debug-check there is only one flow
            let mut ids = pattern.identifiers();
            debug_assert!(ids.len() == 1);
            let id_pattern = ids.pop().unwrap();

            // get the source id, debug-check there is only one flow
            debug_assert!(dependencies.len() == 1);
            let id_source = dependencies.pop().unwrap();
            let source_name = self.ctx0.get_name(id_source);

            let timer_id = self.stmts_timers[&stmt_id].clone();

            let occurrences = (
                self.events.contains(&id_source),
                self.events.contains(&timer_id),
            );
            let import_flow = self.get_stmt_import(stmt_id);
            let reset = self.reset_timer(timer_id, import_flow);
            let source_instr = |els| {
                // if activated, reset timer
                FlowInstruction::if_activated(vec![source_name.clone()], [], reset, els)
            };
            let mut timer_instr = || {
                // if activated, define timeout event and reset timer
                let unit_expr = Expression::some(Expression::lit(Constant::unit_default()));
                let def = self.define_event(id_pattern, unit_expr);
                let reset = self.reset_timer(timer_id, import_flow);
                FlowInstruction::seq(vec![def, reset])
            };
            match occurrences {
                (true, true) => source_instr(Some(timer_instr())),
                (true, false) => source_instr(None),
                (false, true) => timer_instr(),
                (false, false) => {
                    noErrorDesc!("'timeout' should be activated by either its source or its timer")
                }
            }
        }

        /// Compute the instruction from a throttle expression.
        fn handle_throttle(
            &self,
            pattern: &ir1::stmt::Pattern,
            mut dependencies: Vec<usize>,
            delta: Constant,
        ) -> FlowInstruction {
            // get the id of pattern's flow, debug-check there is only one flow
            let mut ids = pattern.identifiers();
            debug_assert!(ids.len() == 1);
            let id_pattern = ids.pop().unwrap();
            let flow_name = self.get_name(id_pattern);

            // get the source id, debug-check there is only one flow
            debug_assert!(dependencies.len() == 1);
            let id_source = dependencies.pop().unwrap();
            let source_name = self.get_name(id_source);

            // update created signal
            let expr = self.get_signal(id_source);
            FlowInstruction::if_throttle(
                flow_name.clone(),
                source_name.clone(),
                delta,
                FlowInstruction::update_ctx(flow_name.clone(), expr),
            )
        }

        /// Compute the instruction from an on_change expression.
        fn handle_on_change(
            &mut self,
            pattern: &ir1::stmt::Pattern,
            mut dependencies: Vec<usize>,
        ) -> FlowInstruction {
            // get the id of pattern's flow, debug-check there is only one flow
            let mut ids = pattern.identifiers();
            debug_assert!(ids.len() == 1);
            let id_pattern = ids.pop().unwrap();

            // get the source id, debug-check there is only one flow
            debug_assert!(dependencies.len() == 1);
            let id_source = dependencies.pop().unwrap();

            let id_old_event = self.on_change_events[&id_pattern];
            let old_event_name = self.ctx0.get_name(id_old_event);

            // detect changes on signal
            let expr = Expression::some(self.get_signal(id_source));
            let event_def = self.define_event(id_pattern, expr);
            let then = vec![
                FlowInstruction::update_ctx(old_event_name.clone(), self.get_signal(id_source)),
                event_def,
            ];
            FlowInstruction::if_change(
                old_event_name.clone(),
                self.get_signal(id_source),
                FlowInstruction::seq(then),
            )
        }

        /// Compute the instruction from a persist expression.
        fn handle_persist(
            &mut self,
            pattern: &ir1::stmt::Pattern,
            mut dependencies: Vec<usize>,
        ) -> FlowInstruction {
            // get the id of pattern's flow, debug-check there is only one flow
            let mut ids = pattern.identifiers();
            debug_assert!(ids.len() == 1);
            let id_pattern = ids.pop().unwrap();
            let flow_name = self.get_name(id_pattern);

            // get the source id, debug-check there is only one flow
            debug_assert!(dependencies.len() == 1);
            let id_source = dependencies.pop().unwrap();

            // update created signal
            let expr = self.get_event(id_source);
            FlowInstruction::if_change(
                flow_name.clone(),
                self.get_event(id_source),
                FlowInstruction::update_ctx(flow_name.clone(), expr),
            )
        }

        /// Compute the instruction from a merge expression.
        fn handle_merge(
            &mut self,
            pattern: &ir1::stmt::Pattern,
            mut dependencies: Vec<usize>,
        ) -> FlowInstruction {
            // get the id of pattern's flow, debug-check there is only one flow
            let mut ids = pattern.identifiers();
            debug_assert!(ids.len() == 1);
            let id_pattern = ids.pop().unwrap();

            // get the source id, debug-check there is only one flow
            debug_assert!(dependencies.len() == 2);
            let id_source_1 = dependencies.pop().unwrap();
            let event_1 = self.get_name(id_source_1).clone();
            let id_source_2 = dependencies.pop().unwrap();
            let event_2 = self.get_name(id_source_2).clone();

            let expr_1 = self.get_event(id_source_1);
            let instr_1 = self.define_event(id_pattern, expr_1);
            let expr_2 = self.get_event(id_source_2);
            let instr_2 = self.define_event(id_pattern, expr_2);

            let if_event_1 = |els| FlowInstruction::if_activated(vec![event_1], [], instr_1, els);
            let if_event_2 = FlowInstruction::if_activated(vec![event_2], [], instr_2, None);

            match (
                self.events.contains(&id_source_1),
                self.events.contains(&id_source_2),
            ) {
                (true, true) => {
                    // check if first activated, otherwise check if second activated
                    if_event_1(Some(if_event_2))
                }
                (true, false) => {
                    // check if first activated
                    if_event_1(None)
                }
                (false, true) => {
                    // check if second activated
                    if_event_2
                }
                (false, false) => noErrorDesc!("'merge' should be activated by one of its sources"),
            }
        }

        /// Compute the instruction from a time expression.
        fn handle_time(
            &mut self,
            stmt_id: usize,
            pattern: &ir1::stmt::Pattern,
            loc: Loc,
        ) -> FlowInstruction {
            // get the id of pattern's flow, debug-check there is only one flow
            let mut ids = pattern.identifiers();
            debug_assert!(ids.len() == 1);
            let id_pattern = ids.pop().unwrap();
            // get the import flow that triggered ``time``
            let import_flow = self.get_stmt_import(stmt_id);
            let mut import_name = self.get_name(import_flow).clone();

            // retrieve the instant of computation
            import_name.set_span(loc.span);
            let expr = Expression::instant(import_name);
            // define a nex signal
            let def = self.define_signal(id_pattern, expr);
            // update context if needed
            if let Some(update) = self.update_ctx(id_pattern) {
                FlowInstruction::seq(vec![def, update])
            } else {
                def
            }
        }

        /// Compute the instruction from a component call.
        fn handle_component_call(
            &mut self,
            pattern: &ir1::stmt::Pattern,
            component_id: usize,
            inputs: &Vec<(usize, flow::Expr)>,
        ) -> FlowInstruction {
            // get events that might call the component
            let (mut comp_inputs, mut signals, mut events) = (vec![], vec![], vec![]);
            inputs.iter().for_each(|(input_id, flow_expr)| {
                match flow_expr.kind {
                    flow::Kind::Ident { id } => {
                        let input_name = self.get_name(*input_id).clone();
                        if self.get_flow_kind(id).is_event() {
                            if self.events.contains(&id) {
                                let event_name = self.get_name(id).clone();
                                events.push(event_name);
                                let input_expr = self.get_event(id);
                                comp_inputs.push((input_name, input_expr));
                            } else {
                                let input_expr = Expression::none();
                                comp_inputs.push((input_name, input_expr));
                            }
                        } else {
                            let signal_name = self.get_name(id).clone();
                            signals.push(signal_name);
                            let input_expr = self.get_signal(id);
                            comp_inputs.push((input_name, input_expr));
                        }
                    }
                    _ => noErrorDesc!(), // normalized
                }
            });

            // call component with the events and update output signals
            self.call_component(component_id, pattern.clone(), comp_inputs, signals, events)
        }

        /// Compute the instruction from a function call.
        fn handle_function_call(
            &mut self,
            pattern: &ir1::stmt::Pattern,
            function_id: usize,
            inputs: &Vec<(usize, flow::Expr)>,
        ) -> FlowInstruction {
            // get events that might call the component
            let (mut fun_inputs, mut signals) = (vec![], vec![]);
            inputs.iter().for_each(|(_, flow_expr)| {
                match flow_expr.kind {
                    flow::Kind::Ident { id } => {
                        let signal_name = self.get_name(id).clone();
                        signals.push(signal_name);
                        let input_expr = self.get_signal(id);
                        fun_inputs.push(input_expr);
                    }
                    _ => noErrorDesc!(), // normalized
                }
            });

            // call component with the events and update output signals
            self.call_function(function_id, pattern.clone(), fun_inputs, signals)
        }

        /// Add signal definition in current propagation branch.
        fn define_signal(&mut self, signal_id: usize, expr: Expression) -> FlowInstruction {
            let signal_name = self.get_name(signal_id).clone();
            self.signals.insert(signal_id);
            FlowInstruction::def_let(signal_name, expr)
        }

        /// Get signal call expression.
        fn get_signal(&self, signal_id: usize) -> Expression {
            let signal_name = self.get_name(signal_id);
            // if signal not already defined, get from context value
            if !self.signals.contains(&signal_id) {
                Expression::in_ctx(signal_name.clone())
            } else {
                Expression::ident(signal_name.clone())
            }
        }

        /// Add event definition in current propagation branch.
        fn define_event(&mut self, event_id: usize, expr: Expression) -> FlowInstruction {
            let event_name = self.get_name(event_id).clone();
            self.events.insert(event_id);
            FlowInstruction::update_event(event_name, expr)
        }

        /// Add reset timer in current propagation branch.
        fn reset_timer(&self, timer_id: usize, import_flow: usize) -> FlowInstruction {
            let timer_name = self.get_name(timer_id);
            let import_name = self.get_name(import_flow);
            FlowInstruction::reset(timer_name.clone(), import_name.clone())
        }

        /// Get event call expression.
        fn get_event(&self, event_id: usize) -> Expression {
            let event_name = self.get_name(event_id);
            Expression::event(event_name.clone())
        }

        /// Add component call in current propagation branch with outputs update.
        fn call_component(
            &mut self,
            component_id: usize,
            output_pattern: ir1::stmt::Pattern,
            inputs: Vec<(Ident, Expression)>,
            signals: Vec<Ident>,
            events: Vec<Ident>,
        ) -> FlowInstruction {
            let component_name = self.get_name(component_id);
            let outputs_ids = output_pattern.identifiers();

            // call component
            let mut instrs = vec![FlowInstruction::comp_call(
                output_pattern.into_ir2(self),
                component_name.clone(),
                inputs,
            )];
            // update outputs: context signals and all events
            let updates = outputs_ids.into_iter().filter_map(|output_id| {
                if self.get_flow_kind(output_id).is_event() {
                    let event_name = self.get_name(output_id);
                    let expr = Expression::ident(event_name.clone());
                    Some(self.define_event(output_id, expr))
                } else {
                    self.signals.insert(output_id);
                    let expr = self.update_ctx(output_id);
                    self.signals.remove(&output_id);
                    expr
                }
            });
            instrs.extend(updates);
            let comp_call = FlowInstruction::seq(instrs);

            match self.conf.propagation {
                conf::Propagation::EventIsles => comp_call, // call component when activated by isle
                conf::Propagation::OnChange => {
                    // call component when activated by inputs
                    FlowInstruction::if_activated(events, signals, comp_call, None)
                }
            }
        }

        /// Add function call in current propagation branch with outputs update.
        fn call_function(
            &mut self,
            function_id: usize,
            output_pattern: ir1::stmt::Pattern,
            inputs: Vec<Expression>,
            signals: Vec<Ident>,
        ) -> FlowInstruction {
            let function_name = self.get_name(function_id);
            let outputs_ids = output_pattern.identifiers();

            // call component
            let mut instrs = vec![FlowInstruction::fun_call(
                output_pattern.into_ir2(self),
                function_name.clone(),
                inputs,
            )];
            // update outputs: context signals and all events
            let updates = outputs_ids.into_iter().filter_map(|output_id| {
                self.signals.insert(output_id);
                let expr = self.update_ctx(output_id);
                self.signals.remove(&output_id);
                expr
            });
            instrs.extend(updates);
            let fun_call = FlowInstruction::seq(instrs);

            match self.conf.propagation {
                conf::Propagation::EventIsles => fun_call, // call function when activated by isle
                conf::Propagation::OnChange => {
                    // call component when activated by inputs
                    FlowInstruction::if_activated(vec![], signals, fun_call, None)
                }
            }
        }

        /// Add signal send in current propagation branch.
        fn send_signal(&self, signal_id: usize, import_flow: usize) -> FlowInstruction {
            let signal_name = self.get_name(signal_id);
            let expr = self.get_signal(signal_id);
            let send = if self.multiple_inputs {
                FlowInstruction::send(signal_name.clone(), expr, false)
            } else {
                let import_name = self.get_name(import_flow);
                FlowInstruction::send_from(signal_name.clone(), expr, import_name.clone(), false)
            };
            if self
                .events
                .iter()
                .any(|event_id| self.ctx0.is_timeout(*event_id))
            {
                send
            } else {
                FlowInstruction::if_activated([], vec![signal_name.clone()], send, None)
            }
        }

        /// Add event send in current propagation branch.
        fn send_event(&self, event_id: usize, import_flow: usize) -> FlowInstruction {
            // timer is an event, look if it is defined
            if self.events.contains(&event_id) {
                // if activated, send event
                let event_name = self.get_name(event_id);
                let expr = Expression::ident(event_name.clone());
                if self.multiple_inputs {
                    FlowInstruction::send(event_name.clone(), expr, true)
                } else {
                    let import_name = self.get_name(import_flow);
                    FlowInstruction::send_from(event_name.clone(), expr, import_name.clone(), true)
                }
            } else {
                FlowInstruction::seq(vec![])
            }
        }

        /// Add flow send in current propagation branch.
        pub fn send(&self, stmt_id: usize, flow_id: usize) -> FlowInstruction {
            let import_flow = self.get_stmt_import(stmt_id);
            // insert instruction only if source is a signal or an activated event
            if self.get_flow_kind(flow_id).is_signal() {
                self.send_signal(flow_id, import_flow)
            } else {
                self.send_event(flow_id, import_flow)
            }
        }

        /// Add context update in current propagation branch.
        fn update_ctx(&self, flow_id: usize) -> Option<FlowInstruction> {
            // if flow is in context, add context_update instruction
            if self.flows_context.contains_element(self.get_name(flow_id)) {
                let expr: Expression = if self.get_flow_kind(flow_id).is_event() {
                    self.get_event(flow_id)
                } else {
                    self.get_signal(flow_id)
                };
                let flow_name = self.get_name(flow_id);
                Some(FlowInstruction::update_ctx(flow_name.clone(), expr))
            } else {
                None
            }
        }
    }
}

mod from_synced {
    prelude! { just
        BTreeMap as Map,
        ir1::interface::{ FlowStatement, FlowDeclaration, FlowInstantiation },
        execution_machine::{ParaMethod, FlowInstruction},
        synced::generic::{ CtxSpec, Synced },
    }

    use super::flow_instr;

    pub trait FromSynced<Ctx: CtxSpec + ?Sized>: Sized {
        fn prefix(ctx: &mut Ctx) -> Self;
        fn suffix(ctx: &mut Ctx) -> Self;
        fn from_instr(ctx: &mut Ctx, instr: Ctx::Instr) -> Self;
        fn from_seq(ctx: &mut Ctx, seq: Vec<Self>) -> Self;
        fn from_para(ctx: &mut Ctx, para: Map<ParaMethod, Vec<Self>>) -> Self;
    }

    pub trait IntoParaMethod {
        fn into_para_method(self) -> ParaMethod;
    }

    enum Frame<Ctx, Instr>
    where
        Ctx: CtxSpec + ?Sized,
        Ctx::Cost: IntoParaMethod,
        Instr: FromSynced<Ctx>,
    {
        Seq {
            done: Vec<Instr>,
            todo: Vec<Synced<Ctx>>,
        },
        Para {
            done: Map<ParaMethod, Vec<Instr>>,
            method: ParaMethod,
            todo: Map<Ctx::Cost, Vec<Synced<Ctx>>>,
        },
    }

    struct Stack<Ctx, Instr>
    where
        Ctx: CtxSpec + ?Sized,
        Ctx::Cost: IntoParaMethod,
        Instr: FromSynced<Ctx>,
    {
        stack: Vec<Frame<Ctx, Instr>>,
    }

    impl<Ctx, Instr> Stack<Ctx, Instr>
    where
        Ctx: CtxSpec + ?Sized,
        Ctx::Cost: IntoParaMethod,
        Instr: FromSynced<Ctx>,
    {
        fn new() -> Self {
            Self {
                stack: Vec::with_capacity(11),
            }
        }

        fn push(&mut self, frame: Frame<Ctx, Instr>) {
            self.stack.push(frame)
        }

        fn pop(&mut self) -> Option<Frame<Ctx, Instr>> {
            self.stack.pop()
        }
    }

    pub fn run<Ctx, Instr>(ctx: &mut Ctx, synced: Synced<Ctx>) -> Instr
    where
        Ctx: CtxSpec + ?Sized,
        Ctx::Cost: IntoParaMethod,
        Instr: FromSynced<Ctx>,
    {
        let mut stack: Stack<Ctx, Instr> = Stack::new();
        let mut acc = None;
        let mut current = synced;

        'go_down: loop {
            debug_assert!(acc.is_none());
            match current {
                Synced::Instr(instr, _) => acc = Some(Instr::from_instr(ctx, instr)),
                Synced::Seq(mut todo, _) => {
                    todo.reverse();
                    current = todo
                        .pop()
                        .expect("there should be a synced in this sequence");
                    stack.push(Frame::Seq { done: vec![], todo });
                    continue 'go_down;
                }
                Synced::Para(mut todo, _) => {
                    let (cost, mut cost_todo) = todo
                        .pop_first()
                        .expect("there should be synced elements in this parallel execution");
                    current = cost_todo
                        .pop()
                        .expect("there should be a synced in this parallel execution");
                    if !cost_todo.is_empty() {
                        todo.insert(cost.clone(), cost_todo);
                    }
                    stack.push(Frame::Para {
                        done: Map::new(),
                        todo,
                        method: cost.into_para_method(),
                    });
                    continue 'go_down;
                }
            }

            'go_up: loop {
                debug_assert!(acc.is_some());
                match stack.pop() {
                    None => {
                        // prefix ; acc ; suffix
                        let prefix = Instr::prefix(ctx);
                        let instr = acc.expect("there should be an instruction to return");
                        let suffix = Instr::suffix(ctx);
                        return Instr::from_seq(ctx, vec![prefix, instr, suffix]);
                    }
                    Some(Frame::Para {
                        mut done,
                        method,
                        mut todo,
                    }) => {
                        let instr = std::mem::take(&mut acc)
                            .expect("there should be an instruction to parallelize");
                        done.entry(method).or_insert_with(Vec::new).push(instr);
                        if let Some((cost, mut cost_todo)) = todo.pop_first() {
                            current = cost_todo
                                .pop()
                                .expect("impossible: `if !cost_todo.is_empty() {`");
                            if !cost_todo.is_empty() {
                                todo.insert(cost.clone(), cost_todo);
                            }
                            stack.push(Frame::Para {
                                done,
                                todo,
                                method: cost.into_para_method(),
                            });
                            continue 'go_down;
                        } else {
                            acc = Some(Instr::from_para(ctx, done));
                            continue 'go_up;
                        }
                    }
                    Some(Frame::Seq { mut done, mut todo }) => {
                        let instr = std::mem::take(&mut acc)
                            .expect("there should be an instruction to sequence");
                        done.push(instr);
                        if let Some(current_todo) = todo.pop() {
                            current = current_todo;
                            stack.push(Frame::Seq { done, todo });
                            continue 'go_down;
                        } else {
                            acc = Some(Instr::from_seq(ctx, done));
                            continue 'go_up;
                        }
                    }
                }
            }
        }
    }

    impl<'a> CtxSpec for flow_instr::Builder<'a> {
        type Instr = usize;
        type Cost = usize;
        type Label = ();
        fn ignore_edge(_: &Self::Label) -> bool {
            false
        }
        fn instr_cost(&self, _i: Self::Instr) -> Self::Cost {
            1 // todo: nb of expressions used in component
        }
        fn sync_seq_cost(&self, seq: &[Synced<Self>]) -> Self::Cost {
            seq.iter().map(Synced::cost).sum()
        }
        fn sync_para_cost(&self, map: &Map<Self::Cost, Vec<Synced<Self>>>) -> Self::Cost {
            let mut max = 0;
            for c in map.keys() {
                max = std::cmp::max(max, *c);
            }
            max + 1
        }
    }
    impl<'a> IntoParaMethod for <flow_instr::Builder<'a> as CtxSpec>::Cost {
        fn into_para_method(self) -> ParaMethod {
            ParaMethod::Tokio // todo: depending on benchmarks
        }
    }
    impl<'a> FromSynced<flow_instr::Builder<'a>> for FlowInstruction {
        fn from_instr(
            ctx: &mut flow_instr::Builder,
            instr: <flow_instr::Builder as CtxSpec>::Instr,
        ) -> Self {
            // get flow statement related to instr
            if let Some(flow_statement) = ctx.get_stmt(instr) {
                match flow_statement.clone() {
                    FlowStatement::Declaration(FlowDeclaration { pattern, expr, .. })
                    | FlowStatement::Instantiation(FlowInstantiation { pattern, expr, .. }) => {
                        ctx.handle_expr(instr, &pattern, &expr)
                    }
                }
            } else
            // get flow export related to instr
            if let Some(export) = ctx.get_export(instr) {
                ctx.send(instr, export.id)
            } else
            // get flow import related to instr
            if let Some(import) = ctx.get_import(instr) {
                ctx.handle_import(import.id)
            } else {
                crate::prelude::noErrorDesc!()
            }
        }

        fn from_seq(_ctx: &mut flow_instr::Builder, seq: Vec<Self>) -> Self {
            FlowInstruction::seq(seq)
        }

        fn from_para(_ctx: &mut flow_instr::Builder, para: Map<ParaMethod, Vec<Self>>) -> Self {
            FlowInstruction::para(para)
        }

        fn prefix(ctx: &mut flow_instr::Builder<'a>) -> Self {
            // init events that should be declared as &mut.
            let init_events = ctx.init_events().collect::<Vec<_>>();
            FlowInstruction::seq(init_events)
        }

        fn suffix(_ctx: &mut flow_instr::Builder<'a>) -> Self {
            FlowInstruction::seq(vec![])
        }
    }
}

mod clean_synced {
    use super::flow_instr;

    prelude! { just
        BTreeMap as Map,
        synced::generic::{ CtxSpec, Synced },
    }

    pub trait IsExport: CtxSpec {
        fn is_export(ctx: &Self, instr: Self::Instr) -> bool;
    }

    enum Frame<Ctx>
    where
        Ctx: CtxSpec + ?Sized,
    {
        Seq {
            done: Vec<Synced<Ctx>>,
            todo: Vec<Synced<Ctx>>,
        },
    }

    struct Stack<Ctx>
    where
        Ctx: CtxSpec + ?Sized,
    {
        stack: Vec<Frame<Ctx>>,
    }

    impl<Ctx> Stack<Ctx>
    where
        Ctx: CtxSpec + ?Sized,
    {
        fn new() -> Self {
            Self {
                stack: Vec::with_capacity(11),
            }
        }

        fn push(&mut self, frame: Frame<Ctx>) {
            self.stack.push(frame)
        }

        fn pop(&mut self) -> Option<Frame<Ctx>> {
            self.stack.pop()
        }
    }

    /// Puts the exports out of parallel instructions.
    pub fn run<Ctx>(ctx: &Ctx, synced: Synced<Ctx>) -> Synced<Ctx>
    where
        Ctx: CtxSpec + IsExport + ?Sized,
    {
        let mut stack: Stack<Ctx> = Stack::new();
        let mut acc = None;
        let mut current = synced;

        'go_down: loop {
            debug_assert!(acc.is_none());
            match current {
                Synced::Instr(_, _) => acc = Some(current),
                Synced::Seq(mut todo, _) => {
                    todo.reverse();
                    current = todo
                        .pop()
                        .expect("there should be a synced in this sequence");
                    stack.push(Frame::Seq { done: vec![], todo });
                    continue 'go_down;
                }
                Synced::Para(_, _) => {
                    acc = Some(extract_exports(ctx, current));
                }
            }

            'go_up: loop {
                debug_assert!(acc.is_some());
                match stack.pop() {
                    None => {
                        let synced = acc.expect("there should be a synced to return");
                        return synced;
                    }
                    Some(Frame::Seq { mut done, mut todo }) => {
                        let instr = std::mem::take(&mut acc)
                            .expect("there should be an instruction to sequence");
                        done.push(instr);
                        if let Some(current_todo) = todo.pop() {
                            current = current_todo;
                            stack.push(Frame::Seq { done, todo });
                            continue 'go_down;
                        } else {
                            acc = Some(Synced::seq(done, ctx));
                            continue 'go_up;
                        }
                    }
                }
            }
        }
    }

    enum ExtractFrame<Ctx>
    where
        Ctx: CtxSpec + ?Sized,
    {
        Seq {
            done: Vec<Synced<Ctx>>,
            todo: Vec<Synced<Ctx>>,
        },
        Para {
            done: Map<Ctx::Cost, Vec<Synced<Ctx>>>,
            cost: Ctx::Cost,
            todo: Map<Ctx::Cost, Vec<Synced<Ctx>>>,
        },
    }

    struct ExtractStack<Ctx>
    where
        Ctx: CtxSpec + ?Sized,
    {
        stack: Vec<ExtractFrame<Ctx>>,
    }
    impl<Ctx> ExtractStack<Ctx>
    where
        Ctx: CtxSpec + ?Sized,
    {
        fn new() -> Self {
            Self {
                stack: Vec::with_capacity(11),
            }
        }

        fn push(&mut self, frame: ExtractFrame<Ctx>) {
            self.stack.push(frame)
        }

        fn pop(&mut self) -> Option<ExtractFrame<Ctx>> {
            self.stack.pop()
        }
    }

    /// Extracts exports from synced.
    ///
    /// Returns a tuple `(new_synced, exports)` where `new_synced` is a copy
    /// of input `synced` without exports, which are in `exports`.
    fn extract_exports<Ctx>(ctx: &Ctx, synced: Synced<Ctx>) -> Synced<Ctx>
    where
        Ctx: CtxSpec + IsExport + ?Sized,
    {
        let mut stack: ExtractStack<Ctx> = ExtractStack::new();
        let mut acc = None;
        let mut exports = vec![];
        let mut current = synced;

        'go_down: loop {
            debug_assert!(acc.is_none());
            match current {
                Synced::Instr(instr, _) => {
                    if IsExport::is_export(ctx, instr) {
                        exports.push(current)
                    } else {
                        acc = Some(current)
                    }
                }
                Synced::Seq(mut todo, _) => {
                    todo.reverse();
                    current = todo
                        .pop()
                        .expect("there should be a synced in this sequence");
                    stack.push(ExtractFrame::Seq { done: vec![], todo });
                    continue 'go_down;
                }
                Synced::Para(mut todo, _) => {
                    let (cost, mut cost_todo) = todo
                        .pop_first()
                        .expect("there should be synced elements in this parallel execution");
                    current = cost_todo
                        .pop()
                        .expect("there should be a synced in this parallel execution");
                    if !cost_todo.is_empty() {
                        todo.insert(cost.clone(), cost_todo);
                    }
                    stack.push(ExtractFrame::Para {
                        done: Map::new(),
                        todo,
                        cost,
                    });
                    continue 'go_down;
                }
            }

            'go_up: loop {
                match stack.pop() {
                    None => {
                        if let Some(para) = acc {
                            if exports.is_empty() {
                                return para;
                            } else {
                                let exports = Synced::seq(exports, ctx);
                                return Synced::seq(vec![para, exports], ctx);
                            }
                        } else {
                            debug_assert!(
                                !exports.is_empty(),
                                "otherwise, the input 'synced' is empty"
                            );
                            return Synced::seq(exports, ctx);
                        }
                    }
                    Some(ExtractFrame::Para {
                        mut done,
                        cost,
                        mut todo,
                    }) => {
                        if let Some(instr) = std::mem::take(&mut acc) {
                            done.entry(cost).or_insert_with(Vec::new).push(instr);
                        }
                        if let Some((cost, mut cost_todo)) = todo.pop_first() {
                            current = cost_todo
                                .pop()
                                .expect("impossible: `if !cost_todo.is_empty() {`");
                            if !cost_todo.is_empty() {
                                todo.insert(cost.clone(), cost_todo);
                            }
                            stack.push(ExtractFrame::Para { done, todo, cost });
                            continue 'go_down;
                        } else if done.is_empty() {
                            continue 'go_up;
                        } else {
                            acc = Some(Synced::para(done, ctx));
                            continue 'go_up;
                        }
                    }
                    Some(ExtractFrame::Seq { mut done, mut todo }) => {
                        if let Some(instr) = std::mem::take(&mut acc) {
                            done.push(instr);
                        }
                        if let Some(current_todo) = todo.pop() {
                            current = current_todo;
                            stack.push(ExtractFrame::Seq { done, todo });
                            continue 'go_down;
                        } else if done.is_empty() {
                            continue 'go_up;
                        } else {
                            acc = Some(Synced::seq(done, ctx));
                            continue 'go_up;
                        }
                    }
                }
            }
        }
    }

    impl<'a> IsExport for flow_instr::Builder<'a> {
        fn is_export(ctx: &Self, instr: Self::Instr) -> bool {
            ctx.get_export(instr).is_some()
        }
    }
}
