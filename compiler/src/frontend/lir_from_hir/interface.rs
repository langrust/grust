use itertools::Itertools;
use petgraph::{algo::toposort, Direction};
use std::collections::HashSet;

prelude! {
    quote::format_ident,
    hir::{
        flow, IdentifierCreator, interface::{
            FlowDeclaration, FlowExport, FlowImport, FlowInstantiation,
            FlowStatement, Interface, Service,
        },
    },
    lir::item::execution_machine::{
        ArrivingFlow,
        flows_context::FlowsContext,
        service_handler::{
            Expression, FlowHandler, FlowInstruction, ServiceHandler,
        },
        runtime_loop::{InputHandler, RuntimeLoop},
        ExecutionMachine,TimingEvent, TimingEventKind, InterfaceFlow,
    },
}

use super::LIRFromHIR;

impl Interface {
    pub fn lir_from_hir(self, symbol_table: &mut SymbolTable) -> ExecutionMachine {
        if self.services.is_empty() {
            return Default::default();
        }
        let Interface {
            mut imports,
            exports,
            services,
        } = self;
        let mut timing_events = vec![];

        let services_handlers: Vec<ServiceHandler> = services
            .into_iter()
            .map(|service| {
                service.lir_from_hir(&mut imports, &exports, &mut timing_events, symbol_table)
            })
            .collect();

        let input_handlers = services_handlers
            .iter()
            .map(|service_handler| {
                service_handler.flows_handling.iter().map(|flow_handler| {
                    (&flow_handler.arriving_flow, service_handler.service.clone())
                })
            })
            .flatten()
            .group_by(|(arriving_flow, _)| *arriving_flow)
            .into_iter()
            .map(|(arriving_flow, services)| InputHandler {
                arriving_flow: (*arriving_flow).clone(),
                services: services.map(|(_, services)| services).collect(),
            })
            .collect();
        let input_flows = imports
            .into_values()
            .filter_map(|import| import.lir_from_hir(symbol_table))
            .collect();
        let output_flows = exports
            .into_values()
            .map(|export| export.lir_from_hir(symbol_table))
            .collect();

        let runtime_loop = RuntimeLoop { input_handlers };

        ExecutionMachine {
            runtime_loop,
            services_handlers,
            input_flows,
            output_flows,
            timing_events,
        }
    }
}

impl LIRFromHIR for FlowImport {
    type LIR = Option<InterfaceFlow>;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        let FlowImport {
            id,
            path,
            flow_type,
            ..
        } = self;

        if flow_type.eq(&Typ::event(Typ::time())) {
            None
        } else {
            Some(InterfaceFlow {
                path,
                identifier: symbol_table.get_name(id).clone(),
                r#type: flow_type,
            })
        }
    }
}

impl LIRFromHIR for FlowExport {
    type LIR = InterfaceFlow;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        let FlowExport {
            id,
            path,
            flow_type,
            ..
        } = self;

        InterfaceFlow {
            path,
            identifier: symbol_table.get_name(id).clone(),
            r#type: flow_type,
        }
    }
}

impl Service {
    pub fn lir_from_hir(
        self,
        imports: &mut HashMap<usize, FlowImport>,
        exports: &HashMap<usize, FlowExport>,
        timing_events: &mut Vec<TimingEvent>,
        symbol_table: &mut SymbolTable,
    ) -> ServiceHandler {
        self.get_service_handler(imports, exports, timing_events, symbol_table)
    }

    fn get_flows_context(&self, symbol_table: &SymbolTable) -> FlowsContext {
        let mut flows_context = FlowsContext {
            elements: Default::default(),
            components: Default::default(),
        };
        self.statements
            .values()
            .for_each(|statement| statement.add_flows_context(&mut flows_context, symbol_table));
        flows_context
    }

    fn get_service_handler(
        mut self,
        imports: &mut HashMap<usize, FlowImport>,
        exports: &HashMap<usize, FlowExport>,
        timing_events: &mut Vec<TimingEvent>,
        symbol_table: &mut SymbolTable,
    ) -> ServiceHandler {
        symbol_table.local();
        let mut flows_context = self.get_flows_context(symbol_table);
        let mut identifier_creator = IdentifierCreator::from(self.get_flows_names(symbol_table));

        // collects components, input flows, output flows, timing events that are present in the service
        let mut components = vec![];
        let mut stmts_timers = HashMap::new();
        let mut on_change_events = HashMap::new();
        self.statements.iter().for_each(|(stmt_id, statement)| {
            let stmt_id = *stmt_id;
            match statement {
                FlowStatement::Declaration(FlowDeclaration {
                    pattern,
                    flow_expression,
                    ..
                })
                | FlowStatement::Instantiation(FlowInstantiation {
                    pattern,
                    flow_expression,
                    ..
                }) => {
                    match &flow_expression.kind {
                        flow::Kind::Ident { .. }
                        | flow::Kind::Throttle { .. }
                        | flow::Kind::Merge { .. } => (),
                        flow::Kind::OnChange { .. } => {
                            // get the identifier of the created event
                            let mut ids = pattern.identifiers();
                            debug_assert!(ids.len() == 1);
                            let flow_event_id = ids.pop().unwrap();
                            let event_name = symbol_table.get_name(flow_event_id).clone();

                            // add new event into the identifier creator
                            let fresh_name =
                                identifier_creator.new_identifier_with("", &event_name, "old");
                            let typing = symbol_table.get_type(flow_event_id).clone();
                            let kind = symbol_table.get_flow_kind(flow_event_id).clone();
                            let fresh_id = symbol_table.insert_fresh_flow(
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
                            let fresh_name = identifier_creator.fresh_identifier("period");
                            let typing = Typ::Event(Box::new(Typ::Time));
                            let fresh_id =
                                symbol_table.insert_fresh_period(fresh_name.clone(), *period_ms);

                            // add timing_event in imports
                            let fresh_statement_id = symbol_table.get_fresh_id();
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
                            self.graph.add_edge(fresh_statement_id, stmt_id, ());

                            // push timing_event
                            stmts_timers.insert(stmt_id, fresh_id);
                            timing_events.push(TimingEvent {
                                identifier: fresh_name,
                                kind: TimingEventKind::Period(period_ms.clone()),
                            });
                        }
                        flow::Kind::Timeout { deadline, .. } => {
                            // add new timing event into the identifier creator
                            let fresh_name = identifier_creator.fresh_identifier("timeout");
                            let typing = Typ::Event(Box::new(Typ::Time));
                            let fresh_id =
                                symbol_table.insert_fresh_deadline(fresh_name.clone(), *deadline);

                            // add timing_event in imports
                            let fresh_statement_id = symbol_table.get_fresh_id();
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
                            self.graph.add_edge(fresh_statement_id, stmt_id, ());

                            // push timing_event
                            stmts_timers.insert(stmt_id, fresh_id);
                            timing_events.push(TimingEvent {
                                identifier: fresh_name,
                                kind: TimingEventKind::Timeout(deadline.clone()),
                            })
                        }
                        flow::Kind::ComponentCall { component_id, .. } => {
                            // add potential period constrains
                            if let Some(period) = symbol_table.get_node_period(*component_id) {
                                // add new timing event into the identifier creator
                                let fresh_name = identifier_creator.fresh_identifier("period");
                                let typing = Typ::Event(Box::new(Typ::Time));
                                let fresh_id =
                                    symbol_table.insert_fresh_period(fresh_name.clone(), period);
                                symbol_table.set_node_period_id(*component_id, fresh_id);

                                // add timing_event in imports
                                let fresh_statement_id = symbol_table.get_fresh_id();
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
                                self.graph.add_edge(fresh_statement_id, stmt_id, ());

                                // push timing_event
                                stmts_timers.insert(stmt_id, fresh_id);
                                timing_events.push(TimingEvent {
                                    identifier: fresh_name,
                                    kind: TimingEventKind::Period(period.clone()),
                                })
                            }
                            components.push(symbol_table.get_name(*component_id).clone())
                        }
                    }
                }
            };
        });

        // create flow propagations
        let mut propag_builder = PropagationBuilder::new(
            &self,
            symbol_table,
            &flows_context,
            imports,
            exports,
            on_change_events,
            stmts_timers,
        );
        propag_builder.propagate();
        let propagations = propag_builder.into_propagations();

        // create flow handlers according to propagations
        let flows_handling: Vec<_> = propagations.into_flow_handlers(symbol_table).collect();

        symbol_table.global();
        ServiceHandler {
            service: symbol_table.get_name(self.id).to_string(),
            components,
            flows_handling,
            flows_context,
        }
    }

    fn is_comp_call(&self, id: usize) -> bool {
        self.statements
            .get(&id)
            .map_or(false, FlowStatement::is_comp_call)
    }
}

impl FlowStatement {
    fn add_flows_context(&self, flows_context: &mut FlowsContext, symbol_table: &SymbolTable) {
        match self {
            FlowStatement::Declaration(FlowDeclaration {
                pattern,
                flow_expression,
                ..
            })
            | FlowStatement::Instantiation(FlowInstantiation {
                pattern,
                flow_expression,
                ..
            }) => match &flow_expression.kind {
                flow::Kind::Throttle { .. } => {
                    // get the id of pattern's flow (and check their is only one flow)
                    let mut ids = pattern.identifiers();
                    debug_assert!(ids.len() == 1);
                    let pattern_id = ids.pop().unwrap();

                    // push in signals context
                    let flow_name = symbol_table.get_name(pattern_id).clone();
                    let ty = symbol_table.get_type(pattern_id);
                    flows_context.add_element(flow_name, ty);
                }
                flow::Kind::Sample {
                    flow_expression, ..
                } => {
                    // get the id of flow_expression (and check it is an idnetifier, from normalization)
                    let id = match &flow_expression.kind {
                        flow::Kind::Ident { id } => *id,
                        _ => unreachable!(),
                    };
                    // get pattern's id
                    let mut ids = pattern.identifiers();
                    assert!(ids.len() == 1);
                    let pattern_id = ids.pop().unwrap();

                    // push in signals context
                    let source_name = symbol_table.get_name(id).clone();
                    let flow_name = symbol_table.get_name(pattern_id).clone();
                    let ty = Typ::SMEvent(Box::new(symbol_table.get_type(id).clone()));
                    flows_context.add_element(source_name, &ty);
                    flows_context.add_element(flow_name, &ty);
                }
                flow::Kind::Scan {
                    flow_expression, ..
                } => {
                    // get the id of flow_expression (and check it is an idnetifier, from normalization)
                    let id = match &flow_expression.kind {
                        flow::Kind::Ident { id } => *id,
                        _ => unreachable!(),
                    };

                    // push in signals context
                    let source_name = symbol_table.get_name(id).clone();
                    let ty = symbol_table.get_type(id);
                    flows_context.add_element(source_name, ty);
                }
                flow::Kind::ComponentCall {
                    component_id,
                    inputs,
                } => {
                    // get outputs' ids
                    let outputs_ids = pattern.identifiers();

                    // store output signals in flows_context
                    for output_id in outputs_ids.iter() {
                        let output_name = symbol_table.get_name(*output_id);
                        let output_type = symbol_table.get_type(*output_id);
                        flows_context.add_element(output_name.clone(), output_type)
                    }

                    let mut events_fields = vec![];
                    let mut signals_fields = vec![];
                    inputs.iter().for_each(|(input_id, flow_expression)| {
                        match &flow_expression.kind {
                            // get the id of flow_expression (and check it is an idnetifier, from normalization)
                            flow::Kind::Ident { id: flow_id } => {
                                let input_field_name = symbol_table.get_name(*input_id).clone();
                                let flow_name = symbol_table.get_name(*flow_id).clone();
                                let ty = symbol_table.get_type(*flow_id);
                                if ty.is_event() {
                                    // push in events_fields if event
                                    events_fields.push((input_field_name, flow_name, ty.clone()));
                                } else {
                                    // push in signals_fields if signal
                                    signals_fields.push((input_field_name, flow_name.clone()));
                                    // push in context
                                    flows_context.add_element(flow_name, ty);
                                }
                            }
                            _ => unreachable!(),
                        }
                    });

                    flows_context.add_component(
                        symbol_table.get_name(*component_id).clone(),
                        events_fields,
                        signals_fields,
                    )
                }
                flow::Kind::Ident { .. }
                | flow::Kind::OnChange { .. }
                | flow::Kind::Timeout { .. }
                | flow::Kind::Merge { .. } => (),
            },
        }
    }
}

/// An *"isle"* for some event `e` is all (and only) the statements to run when receiving `e`.
///
/// This structure is only meant to be used immutably *after* it is created by [`IsleBuilder`], in
/// fact all `&mut self` functions on [`Isles`] are private.
///
/// Given a service and some statements, each event triggers statements that feature call to
/// eventful component. To actually run them, we need to update their inputs, which means that we
/// need to know the event-free statements (including event-free component calls) that produce the
/// inputs for each eventful component call triggered.
///
/// The *"isle"* for event `e` is the list of statements from the service that need to run to
/// fully compute the effect of receiving `e` (including top stateful calls). The isle is a sub-list
/// of the original list of statements, in particular it is ordered the same way.
pub struct Isles {
    /// Maps event indices to node isles.
    events_to_isles: HashMap<usize, Vec<usize>>,
}
impl Isles {
    /// Constructor for an event capacity.
    pub fn with_capacity(event_capa: usize) -> Self {
        Self {
            events_to_isles: HashMap::with_capacity(event_capa),
        }
    }

    /// Turns itself into a map from events to their isle.
    ///
    /// This does not compute anything, it just yields the internal map.
    pub fn destroy(self) -> HashMap<usize, Vec<usize>> {
        self.events_to_isles
    }

    /// The statements to run for a given event.
    pub fn get_isle_for(&self, event: usize) -> Option<&Vec<usize>> {
        self.events_to_isles.get(&event)
    }

    /// True if the isle for `event` is empty.
    pub fn is_isle_empty(&self, event: usize) -> bool {
        self.events_to_isles
            .get(&event)
            .map(|isle| isle.is_empty())
            .unwrap_or(true)
    }

    /// Checks that all isles are sorted.
    pub fn check(&self) -> Result<(), &'static str> {
        if !self
            .events_to_isles
            .iter()
            .all(|(_, isle)| isle.iter().check_sorted())
        {
            return Err("[Isles] illegal state, synchronous isles are not sorted");
        }
        Ok(())
    }

    /// Finalizes an isle by sorting (and shrinking) it.
    fn finalize_isle(&mut self, event: usize) {
        let _ = self.events_to_isles.get_mut(&event).map(|isle| {
            isle.sort();
            isle.shrink_to_fit();
        });
    }

    /// Inserts a statement for an event.
    ///
    /// Note that the statements in the isles are not (insert-)sorted by this function, that's why
    /// it is private. Each isle is populated by [`IsleBuilder::trace_event`], which does sort
    /// the isle it creates before returning.
    fn insert(&mut self, event: usize, stmt: usize) {
        self.events_to_isles
            .entry(event)
            .or_insert_with(|| Vec::with_capacity(10))
            .push(stmt)
    }
}

/// A context to build [`Isles`].
///
/// The [constructor][Self::new] requires an [`Service`] (and other things) and **will use its
/// internal [graph][Service::graph], make sure it is properly setup.
pub struct IsleBuilder<'a> {
    /// Result isles, populated during event-based traversals.
    isles: Isles,
    /// Events currently triggered during a traversal.
    ///
    /// This is not used currently, as I'm still not sure the analysis should deal with multiple
    /// events triggering at the same time as the language does not allow it.
    events: HashSet<usize>,
    /// Cached stack of statements to visit.
    ///
    /// The `bool` flag indicates the statement is at top-level, meaning it should be inserted in
    /// the isle despite being stateful.
    stack: Vec<(usize, bool)>,
    /// Cached memory of the statements visited when tracing an event.
    memory: HashSet<usize>,
    /// Maps event indices to the (indices of) statements triggered by this event.
    event_to_stmts: HashMap<usize, Vec<usize>>,
    /// Symbol table.
    syms: &'a SymbolTable,
    /// Service to build isles for.
    service: &'a Service,
}
impl<'a> IsleBuilder<'a> {
    /// Factored [`Isles`] allocation.
    fn new_isles(_syms: &'a SymbolTable) -> Isles {
        // #TODO retrieve event count from `syms` for capacity
        Isles::with_capacity(10)
    }

    /// Constructor.
    ///
    /// The `service`'s [graph][Service::graph] must be properly setup for the builder to work.
    ///
    /// During construction, the statements of the `service` are scanned to populate a map from
    /// events to the statements that react to it.
    pub fn new(syms: &'a SymbolTable, service: &'a Service) -> Self {
        let event_to_stmts = Self::build_event_to_stmts(syms, service);
        Self {
            isles: Self::new_isles(syms),
            events: HashSet::with_capacity(10),
            stack: Vec::with_capacity(service.statements.len() / 2),
            memory: HashSet::with_capacity(service.statements.len()),
            event_to_stmts,
            syms,
            service,
        }
    }

    /// Scans the statements in the `service` and produces the map from events to statements.
    ///
    /// Used by [`Self::new`].
    ///
    /// The vector of statement indices associated to any event identifier is **sorted**, *i.e.*
    /// statement indices are in the order in which they appear in the service. (It actually does
    /// not matter for the actual isle building process atm.)
    fn build_event_to_stmts(
        syms: &SymbolTable,
        service: &'a Service,
    ) -> HashMap<usize, Vec<usize>> {
        let mut map = HashMap::with_capacity(10);
        for (stmt_id, stmt) in service.statements.iter() {
            let mut triggered_by = |event: usize| {
                let vec = map.entry(event).or_insert_with(Vec::new);
                debug_assert!(!vec.contains(stmt_id));
                vec.push(*stmt_id);
            };

            if let Some((comp, inputs)) = stmt.try_get_call() {
                if let Some(timer) = syms.get_node_period_id(comp) {
                    // register `stmt_id` as triggered by `timer`
                    triggered_by(timer);
                }
                // scan inputs for events
                for input in inputs {
                    if let flow::Kind::Ident { id: input } = input.1.kind {
                        if syms.get_flow_kind(input).is_event() {
                            // register `stmt_id` as triggered by `input`
                            triggered_by(input);
                        }
                    } else {
                        todo!("non-ident component input")
                    }
                }
            }
        }
        // all vectors in `map` should be sorted and non-empty
        debug_assert! {
            map.iter().all(|(_, vec)| !vec.is_empty() && vec.iter().check_sorted())
        }
        map
    }

    /// True if `stmt` corresponds to a component call that reacts to some event.
    ///
    /// Used to stop the exploration of a dependency branch on component calls that are eventful and
    /// not triggered by the event the isle is for.
    fn is_eventful_call(&self, stmt_id: usize) -> bool {
        if let Some(stmt) = self.service.statements.get(&stmt_id) {
            stmt.try_get_call()
                .map(|(id, _)| {
                    self.syms.has_events(id) || self.syms.get_node_period_id(id).is_some()
                })
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// True if `stmt` corresponds to a component call.
    fn is_call(&self, stmt_id: usize) -> bool {
        self.service
            .statements
            .get(&stmt_id)
            .map(FlowStatement::is_comp_call)
            .unwrap_or(false)
    }

    /// Isles accessor.
    pub fn isles(&self) -> &Isles {
        &self.isles
    }

    /// Destroys itself, produces the isles.
    pub fn into_isles(self) -> Isles {
        self.isles
    }

    /// Traces an event to populate the inner [`Isles`].
    pub fn trace_event(&mut self, event: usize) {
        if let Some(stmts_ids) = self.event_to_stmts.get(&event) {
            debug_assert!(self.stack.is_empty());
            // order does not matter that much, we can't be sure to push in the proper order and the
            // finalization in `Self::into_isles` sorts statements anyways
            self.stack
                .extend(stmts_ids.iter().map(|stmt_id| (*stmt_id, true)));
        } else {
            return ();
        }

        debug_assert!(self.isles.is_isle_empty(event));
        debug_assert!(self.memory.is_empty());
        debug_assert!(self.events.is_empty());
        let _is_new = self.events.insert(event);
        debug_assert!(_is_new);

        'stmts: while let Some((stmt_id, is_top)) = self.stack.pop() {
            let is_new = self.memory.insert(stmt_id);
            // continue if not new or we have reached an eventful call that's not at the top
            if !is_new || !is_top && self.is_eventful_call(stmt_id) {
                continue 'stmts;
            }

            if self.is_call(stmt_id) {
                self.isles.insert(event, stmt_id);
            }

            for stmt_id in self
                .service
                .graph
                .neighbors_directed(stmt_id, Direction::Incoming)
            {
                self.stack.push((stmt_id, false));
            }
        }

        // outro
        debug_assert!(self.stack.is_empty());
        self.events.clear();
        self.memory.clear();
        self.isles.finalize_isle(event);
    }

    /// Traces all events appearing in the the symbol table provided on [creation][Self::new].
    pub fn trace_events(&mut self, events: impl IntoIterator<Item = usize>) {
        for event in events {
            self.trace_event(event)
        }
    }
}

/// Accumulator of instructions that can handle onchange_default branches.
#[derive(Default)]
pub struct Accumulator {
    /// Current instructions block.
    current: Vec<FlowInstruction>,
    /// Informations kept when in 'onchange' branch.
    onchange_block: Option<(usize, String, String, Box<Accumulator>)>,
    /// Informations kept when in 'default' branch.
    default_block: Option<(String, String, Vec<FlowInstruction>, Box<Accumulator>)>,
}
impl Accumulator {
    /// New empty accumulator.
    pub fn new() -> Self {
        Self {
            current: Vec::new(),
            onchange_block: None,
            default_block: None,
        }
    }
    /// New empty accumulator with capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            current: Vec::with_capacity(capacity),
            onchange_block: None,
            default_block: None,
        }
    }
    /// Tells if in 'onchange' branch.
    pub fn is_onchange_block(&self) -> bool {
        self.onchange_block.is_some()
    }
    /// Tells if in 'default' branch.
    pub fn is_default_block(&self) -> bool {
        self.default_block.is_some()
    }
    /// Appends an instruction to the end of the current block.
    pub fn push(&mut self, instr: FlowInstruction) {
        self.current.push(instr)
    }
    /// Switch to a onchange branch.
    pub fn onchange(
        self,
        id_event: usize,
        event_name: impl Into<String>,
        old_event_name: impl Into<String>,
        source_name: impl Into<String>,
    ) -> Self {
        let old_event_name = old_event_name.into();
        let source_name = source_name.into();
        Self {
            current: vec![
                FlowInstruction::update_ctx(
                    old_event_name.clone(),
                    Expression::ident(source_name.clone()),
                ),
                FlowInstruction::def_let(event_name, Expression::ident(source_name.clone())),
            ],
            onchange_block: Some((id_event, old_event_name, source_name, self.into())),
            default_block: None,
        }
    }
    /// Switch to a default branch.
    pub fn default(self) -> (Self, usize) {
        debug_assert!(self.onchange_block.is_some());
        debug_assert!(self.default_block.is_none());
        let (id_event, old_event_name, source_name, original_acc) = self.onchange_block.unwrap();
        (
            Self {
                current: vec![],
                onchange_block: None,
                default_block: Some((old_event_name, source_name, self.current, original_acc)),
            },
            id_event,
        )
    }
    /// Combine an onchange branch and a default branch to an if_change instruction.
    pub fn combine(self) -> Self {
        debug_assert!(self.onchange_block.is_none());
        debug_assert!(self.default_block.is_some());
        let els = self.current;
        let (old_event_name, source_name, then, original_acc) = self.default_block.unwrap();
        let instruction = FlowInstruction::if_change(old_event_name, source_name, then, els);
        let mut accumulator = *original_acc;
        accumulator.push(instruction);
        accumulator
    }
}

/// The *"propagation"* of a flow is all (and only) the instructions to run when receiving it.
#[derive(Default)]
pub struct Propagations {
    /// Maps flow indices to propagation instructions.
    input_flows_propagation: HashMap<usize, Accumulator>,
}
impl Propagations {
    /// Inserts an instruction for a flow.
    pub fn insert(&mut self, flow: usize, instruction: FlowInstruction) {
        self.input_flows_propagation
            .get_mut(&flow)
            .unwrap()
            .push(instruction)
    }
    pub fn init_propagation(&mut self, flow: usize) {
        let _unique = self
            .input_flows_propagation
            .insert(flow, Accumulator::with_capacity(10));
        debug_assert!(_unique.is_none())
    }
    /// Makes t possible to iter on propagations.
    pub fn into_iter(self) -> impl Iterator<Item = (usize, Vec<FlowInstruction>)> {
        self.input_flows_propagation
            .into_iter()
            .map(|(flow, accumulator)| {
                debug_assert!(accumulator.onchange_block.is_none());
                debug_assert!(accumulator.default_block.is_none());
                (flow, accumulator.current)
            })
    }
    /// Tells if in 'onchange' branch.
    pub fn is_onchange_block(&self, flow: usize) -> bool {
        let accumulator = self.input_flows_propagation.get(&flow).unwrap();
        accumulator.onchange_block.is_some()
    }
    /// Tells if in 'default' branch.
    pub fn is_default_block(&self, flow: usize) -> bool {
        let accumulator = self.input_flows_propagation.get(&flow).unwrap();
        accumulator.default_block.is_some()
    }
    /// Switch to a onchange branch.
    pub fn onchange(
        &mut self,
        flow: usize,
        id_event: usize,
        event_name: impl Into<String>,
        old_event_name: impl Into<String>,
        source_name: impl Into<String>,
    ) {
        let accumulator = self.input_flows_propagation.get_mut(&flow).unwrap();
        *accumulator =
            std::mem::take(accumulator).onchange(id_event, event_name, old_event_name, source_name);
    }
    /// Switch to an default branch.
    pub fn default(&mut self, flow: usize) -> usize {
        let accumulator = self.input_flows_propagation.get_mut(&flow).unwrap();
        let (new_acc, id_event) = std::mem::take(accumulator).default();
        *accumulator = new_acc;
        id_event
    }
    /// Combine an onchange branch and a default branch to an if_change instruction.
    pub fn combine(&mut self, flow: usize) {
        let accumulator = self.input_flows_propagation.get_mut(&flow).unwrap();
        *accumulator = std::mem::take(accumulator).combine();
    }
    /// Transforms propagations into flow handlers.
    pub fn into_flow_handlers<'a>(
        self,
        symbol_table: &'a SymbolTable,
    ) -> impl Iterator<Item = FlowHandler> + 'a {
        // for every propagation of incoming flows, create their handlers
        self.into_iter().map(|(flow_id, mut instructions)| {
            // determine weither this arriving flow is a timing event
            let flow_name = symbol_table.get_name(flow_id).clone();
            let arriving_flow = if let Some(period) = symbol_table.get_period(flow_id) {
                // add reset periodic timer
                instructions.push(FlowInstruction::ResetTimer(flow_name.clone(), *period));
                ArrivingFlow::Period(flow_name)
            } else if symbol_table.is_deadline(flow_id) {
                ArrivingFlow::Deadline(flow_name)
            } else {
                let flow_type = symbol_table.get_type(flow_id);
                let path = symbol_table.get_path(flow_id);
                ArrivingFlow::Channel(flow_name, flow_type.clone(), path.clone())
            };
            // get the name of timeout events from reset instructions
            let timers_to_reset = instructions
                .iter()
                .filter_map(|instruction| match instruction {
                    FlowInstruction::ResetTimer(deadline_name, _) => Some(deadline_name.clone()),
                    _ => None,
                })
                .collect();
            FlowHandler {
                arriving_flow,
                deadline_args: timers_to_reset,
                instructions,
            }
        })
    }
}

/// Stack of statements indices that can handle forks.
#[derive(Default)]
pub struct Stack {
    /// Current statements stack.
    current: Vec<usize>,
    /// Next statements stack.
    next: Option<Box<Stack>>,
}
impl Stack {
    /// New empty stack.
    pub fn new() -> Self {
        Self {
            current: Vec::new(),
            next: None,
        }
    }
    /// New empty stack with capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            current: Vec::with_capacity(capacity),
            next: None,
        }
    }
    /// Tells if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.current.is_empty() && self.next.as_ref().map_or(true, |stack| stack.is_empty())
    }
    /// Appends a statement identifier to the end of the current stack.
    pub fn push(&mut self, stmt_id: usize) {
        self.current.push(stmt_id)
    }
    /// Pop the next statement identifier from the current stack.
    pub fn pop(&mut self) -> Option<usize> {
        self.current.pop()
    }
    /// Fork the stack.
    pub fn fork(&mut self) {
        let old = std::mem::take(self);
        *self = Self {
            current: old.current.clone(),
            next: Some(old.into()),
        }
    }
    /// Switch to the next stack.
    pub fn next(&mut self) {
        debug_assert!(self.current.is_empty());
        debug_assert!(self.next.is_some());
        *self = *std::mem::take(self).next.unwrap()
    }
    /// Insert in the stack in dependencies order.
    pub fn insert_ordered<F: Fn(usize) -> usize>(&mut self, value: usize, f: F) {
        match self
            .current
            .binary_search_by_key(&f(value), |stmt_id| f(*stmt_id))
        {
            Err(pos) => self.current.insert(pos, value),
            Ok(_) => (), // already in the stack
        }
    }
    /// Extend the stack in dependencies order.
    pub fn extend_ordered(
        &mut self,
        iter: impl Iterator<Item = usize>,
        compare: impl Fn(usize) -> usize + Clone,
    ) {
        iter.for_each(|next_statement_id| {
            // insert statements into the sorted stack
            self.insert_ordered(next_statement_id, compare.clone())
        })
    }
}

/// A context to build [Propagations] of input flows.
pub struct PropagationBuilder<'a> {
    /// Result propagations, populated during traversals.
    propagations: Propagations,
    /// Context of the service.
    flows_context: &'a FlowsContext,
    /// Map from id to import.
    imports: &'a HashMap<usize, FlowImport>,
    /// Map from id to export.
    exports: &'a HashMap<usize, FlowExport>,
    /// Events' isles.
    isles: Isles,
    /// Symbol table.
    symbol_table: &'a SymbolTable,
    /// Service to build isles for.
    service: &'a Service,
    /// Cached indice of incoming flow.
    incoming_flow: usize,
    /// Cached stack of statements to visit.
    stack: Stack,
    /// Cached memory of the statements visited.
    memory: HashSet<usize>,
    /// Events currently triggered during a traversal.
    events: HashSet<usize>,
    /// Signals currently defined during a traversal.
    signals: HashSet<usize>,
    /// The dependency order the statements should follow.
    statements_order: HashMap<usize, usize>,
    /// Maps on_change event indices to the indices of signals containing their previous values.
    on_change_events: HashMap<usize, usize>,
    /// Maps statement indices to the indices and kinds of their timing_events.
    stmts_timers: HashMap<usize, usize>,
}

impl<'a> PropagationBuilder<'a> {
    /// Create a PropagationBuilder.
    ///
    /// After creating the builder, you only need to [propagate](Self::propagate) the input flows.
    /// This will create the instructions to run when the input flow arrives.
    pub fn new(
        service: &'a Service,
        symbol_table: &'a SymbolTable,
        flows_context: &'a FlowsContext,
        imports: &'a HashMap<usize, FlowImport>,
        exports: &'a HashMap<usize, FlowExport>,
        on_change_events: HashMap<usize, usize>,
        stmts_timers: HashMap<usize, usize>,
    ) -> PropagationBuilder<'a> {
        // create events isles
        let mut isle_builder = IsleBuilder::new(symbol_table, service);
        isle_builder.trace_events(service.get_flows_ids());
        let isles = isle_builder.into_isles();

        // sort statement in dependency order
        let mut ordered_statements = toposort(&service.graph, None).expect("should succeed");
        ordered_statements.reverse();
        let statements_order = ordered_statements
            .into_iter()
            .enumerate()
            .map(|(order, statement_id)| (statement_id, order))
            .collect::<HashMap<_, _>>();

        PropagationBuilder {
            propagations: Default::default(),
            flows_context,
            imports,
            exports,
            isles,
            symbol_table,
            service,
            stack: Stack::new(),
            memory: Default::default(),
            incoming_flow: 0,
            events: Default::default(),
            signals: Default::default(),
            statements_order,
            on_change_events,
            stmts_timers,
        }
    }

    /// Destroy PropagationsBuilder into Propagations.
    pub fn into_propagations(self) -> Propagations {
        self.propagations
    }

    fn get_def_flows(&self, id: usize) -> Vec<usize> {
        if let Some(stmt) = self.service.statements.get(&id) {
            stmt.get_identifiers()
        } else if let Some(import) = self.imports.get(&id) {
            vec![import.id]
        } else {
            vec![]
        }
    }

    /// Extend the stack with the next statements to compute.
    fn extend_with_next(&mut self, parent: usize) {
        // get the flows defined by parent statement
        let parent_flows = self.get_def_flows(parent);

        let dependencies = self
            .service
            .graph
            .neighbors(parent)
            .filter_map(|child| {
                // filter component call because they will appear in isles
                if self.service.is_comp_call(child) {
                    return None;
                }
                Some(child)
            })
            .filter_map(|to_insert| {
                // remove already visited
                if self.memory.contains(&to_insert) {
                    println!("filter dependencies");
                    return None;
                }
                Some(to_insert)
            });

        let isles = parent_flows
            .iter()
            .filter_map(|parent_flow| self.isles.get_isle_for(*parent_flow))
            .flatten()
            .filter_map(|to_insert| {
                // remove already visited
                if self.memory.contains(to_insert) {
                    println!("filter isles");
                    return None;
                }
                Some(*to_insert)
            });

        // extend stack with union of event isle and dependencies
        let to_insert = isles.chain(dependencies).unique();

        // gives the order of statements indices
        let compare = |statement_id| self.statements_order[&statement_id];
        self.stack.extend_ordered(to_insert, compare)
    }

    /// Switch to a onchange branch.
    fn onchange(&mut self, id_event: usize, id_source: usize) {
        let event_name = self.symbol_table.get_name(id_event);
        let source_name = self.get_signal_name(id_source);
        let id_old_event = self.on_change_events[&id_event];
        let old_event_name = self.symbol_table.get_name(id_old_event);
        self.propagations.onchange(
            self.incoming_flow,
            id_event,
            event_name,
            old_event_name,
            source_name,
        );
        self.stack.fork();
        self.events.insert(id_event);
    }
    /// Switch to an default branch.
    fn default(&mut self) -> usize {
        let id_event = self.propagations.default(self.incoming_flow);
        self.stack.next();
        id_event
    }
    /// Combine an default branch and a default branch to an default instruction.
    fn combine(&mut self) {
        self.propagations.combine(self.incoming_flow);
    }

    /// Get the next statement identifier to analyse.
    fn pop_stack(&mut self) -> Option<usize> {
        if let Some(value) = self.stack.pop() {
            let _unique = self.memory.insert(value);
            debug_assert!(_unique);
            return Some(value);
        }
        if self.propagations.is_onchange_block(self.incoming_flow) {
            let id_event = self.default();
            self.events.remove(&id_event);
            return self.pop_stack();
        }
        if self.propagations.is_default_block(self.incoming_flow) {
            self.combine();
            return self.pop_stack();
        }
        if self.stack.is_empty() {
            return None;
        }
        unreachable!()
    }

    /// Compute the instructions propagating the changes of all incoming flows.
    pub fn propagate(&mut self) {
        // for every incoming flows, compute their handlers
        self.imports.iter().for_each(|(import_id, import)| {
            self.incoming_flow = import.id;
            self.propagations.init_propagation(import.id);
            self.memory.clear();
            self.events.clear();
            self.signals.clear();
            self.propagate_import(*import_id)
        });
    }

    /// Compute the instructions propagating the changes of one incoming flow.
    fn propagate_import(&mut self, import_id: usize) {
        debug_assert!(self.stack.is_empty());
        self.stack.push(import_id);

        while let Some(stmt_id) = self.pop_stack() {
            // get flow statement related to stmt_id
            if let Some(flow_statement) = self.service.statements.get(&stmt_id) {
                match flow_statement {
                    FlowStatement::Declaration(FlowDeclaration {
                        pattern,
                        flow_expression,
                        ..
                    })
                    | FlowStatement::Instantiation(FlowInstantiation {
                        pattern,
                        flow_expression,
                        ..
                    }) => self.handle_expr(stmt_id, pattern, flow_expression),
                }
            } else
            // get flow import related to stmt_id
            if let Some(import) = self.imports.get(&stmt_id) {
                if self.symbol_table.get_flow_kind(import.id).is_event() {
                    self.events.insert(import.id);
                } else {
                    self.signals.insert(import.id);
                }
                self.update_ctx(import.id);
            } else
            // get flow export related to stmt_id
            if let Some(export) = self.exports.get(&stmt_id) {
                self.send(export.id)
            }

            self.extend_with_next(stmt_id);
        }
    }

    /// Compute the instructions from an expression flow.
    #[inline]
    fn handle_expr(
        &mut self,
        stmt_id: usize,
        pattern: &hir::Pattern,
        flow_expression: &flow::Expr,
    ) {
        let dependencies = flow_expression.get_dependencies();
        match &flow_expression.kind {
            flow::Kind::Ident { id } => self.handle_ident(pattern, *id),
            flow::Kind::Sample { .. } => self.handle_sample(stmt_id, pattern, dependencies),
            flow::Kind::Scan { .. } => self.handle_scan(stmt_id, pattern, dependencies),
            flow::Kind::Timeout { deadline, .. } => {
                self.handle_timeout(stmt_id, pattern, dependencies, *deadline)
            }
            flow::Kind::Throttle { delta, .. } => {
                self.handle_throttle(pattern, dependencies, delta.clone())
            }
            flow::Kind::OnChange { .. } => self.handle_on_change(pattern, dependencies),
            flow::Kind::Merge { .. } => self.handle_merge(pattern, dependencies),
            flow::Kind::ComponentCall {
                component_id,
                inputs,
            } => self.handle_component_call(pattern, *component_id, inputs),
        }
    }

    /// Compute the instructions from an identifier expression.
    fn handle_ident(&mut self, pattern: &hir::Pattern, id_source: usize) {
        // get the id of pattern's flow, debug-check there is only one flow
        let mut ids = pattern.identifiers();
        debug_assert!(ids.len() == 1);
        let id_pattern = ids.pop().unwrap();

        // insert instruction only if source is a signal or an activated event
        if self.symbol_table.get_flow_kind(id_source).is_signal() {
            let expr = self.get_signal(id_source);
            self.define_signal(id_pattern, expr);
        } else {
            let expr = self.get_event(id_source);
            self.define_event(id_pattern, expr);
        }

        self.update_ctx(id_pattern);
    }

    /// Compute the instructions from a sample expression.
    fn handle_sample(
        &mut self,
        stmt_id: usize,
        pattern: &hir::Pattern,
        mut dependencies: Vec<usize>,
    ) {
        // get the id of pattern's flow, debug-check there is only one flow
        let mut ids = pattern.identifiers();
        debug_assert!(ids.len() == 1);
        let id_pattern = ids.pop().unwrap();
        let flow_name = self.symbol_table.get_name(id_pattern);

        // get the source id, debug-check there is only one flow
        debug_assert!(dependencies.len() == 1);
        let id_source = dependencies.pop().unwrap();
        let source_name = self.symbol_table.get_name(id_source);

        let timer_id = self.stmts_timers[&stmt_id];

        // source is an event, look if it is activated
        if self.events.contains(&id_source) {
            // if activated, store event value
            self.push_instr(FlowInstruction::update_ctx(
                source_name,
                Expression::some(Expression::ident(source_name)),
            ))
        }

        // if timing event is activated
        if self.events.contains(&timer_id) {
            // if activated, update signal by taking from stored event value
            self.push_instr(FlowInstruction::update_ctx(
                flow_name,
                Expression::take_from_ctx(source_name),
            ));
        }
    }

    /// Compute the instructions from a scan expression.
    fn handle_scan(
        &mut self,
        stmt_id: usize,
        pattern: &hir::Pattern,
        mut dependencies: Vec<usize>,
    ) {
        // get the id of pattern's flow, debug-check there is only one flow
        let mut ids = pattern.identifiers();
        debug_assert!(ids.len() == 1);
        let id_pattern = ids.pop().unwrap();
        let flow_name = self.symbol_table.get_name(id_pattern);

        // get the source id, debug-check there is only one flow
        debug_assert!(dependencies.len() == 1);
        let id_source = dependencies.pop().unwrap();
        let source_name = self.symbol_table.get_name(id_source);

        let timer_id = self.stmts_timers[&stmt_id];

        // timer is an event, look if it is activated
        if self.events.contains(&timer_id) {
            // if activated, create event
            self.events.insert(id_pattern);

            // add event creation in instructions
            // source is a signal, look if it is defined
            if self.signals.contains(&id_source) {
                self.push_instr(FlowInstruction::def_let(
                    flow_name,
                    Expression::ident(source_name),
                ))
            } else {
                // if not defined, then get it from the context
                self.push_instr(FlowInstruction::def_let(
                    flow_name,
                    Expression::in_ctx(source_name),
                ))
            }
        }
    }

    /// Compute the instructions from a timeout expression.
    fn handle_timeout(
        &mut self,
        stmt_id: usize,
        pattern: &hir::Pattern,
        mut dependencies: Vec<usize>,
        deadline: u64,
    ) {
        // get the id of pattern's flow, debug-check there is only one flow
        let mut ids = pattern.identifiers();
        debug_assert!(ids.len() == 1);
        let id_pattern = ids.pop().unwrap();

        // get the source id, debug-check there is only one flow
        debug_assert!(dependencies.len() == 1);
        let id_source = dependencies.pop().unwrap();

        let timer_id = self.stmts_timers[&stmt_id].clone();

        let expr = self.get_event(id_source).map(Expression::ok);
        let mut to_reset = expr.is_some();
        self.define_event(id_pattern, expr);

        let expr = self.get_event(timer_id).map(|_| Expression::err());
        to_reset = to_reset || expr.is_some();
        self.define_event(id_pattern, expr);

        self.reset_timer(to_reset, timer_id, deadline)
    }

    /// Compute the instructions from a throttle expression.
    fn handle_throttle(
        &mut self,
        pattern: &hir::Pattern,
        mut dependencies: Vec<usize>,
        delta: Constant,
    ) {
        // get the id of pattern's flow, debug-check there is only one flow
        let mut ids = pattern.identifiers();
        debug_assert!(ids.len() == 1);
        let id_pattern = ids.pop().unwrap();
        let flow_name = self.symbol_table.get_name(id_pattern);

        // get the source id, debug-check there is only one flow
        debug_assert!(dependencies.len() == 1);
        let id_source = dependencies.pop().unwrap();
        let source_name = self.symbol_table.get_name(id_source);

        // update created signal
        let expr = self.get_signal(id_source);
        self.push_instr(FlowInstruction::if_throttle(
            flow_name,
            source_name,
            delta,
            FlowInstruction::update_ctx(flow_name, expr),
        ));
    }

    /// Compute the instructions from an on_change expression.
    fn handle_on_change(&mut self, pattern: &hir::Pattern, mut dependencies: Vec<usize>) {
        // get the id of pattern's flow, debug-check there is only one flow
        let mut ids = pattern.identifiers();
        debug_assert!(ids.len() == 1);
        let id_pattern = ids.pop().unwrap();

        // get the source id, debug-check there is only one flow
        debug_assert!(dependencies.len() == 1);
        let id_source = dependencies.pop().unwrap();

        // initiate the onchange branch (propagation will branch later on the default)
        self.onchange(id_pattern, id_source);
    }

    /// Compute the instructions from a merge expression.
    fn handle_merge(&mut self, pattern: &hir::Pattern, dependencies: Vec<usize>) {
        // get the id of pattern's flow, debug-check there is only one flow
        let mut ids = pattern.identifiers();
        debug_assert!(ids.len() == 1);
        let id_pattern = ids.pop().unwrap();
        let flow_name = self.symbol_table.get_name(id_pattern);

        // get the potential activated event
        let dependencies: HashSet<usize> = dependencies.into_iter().collect();
        let mut overlapping_events = dependencies.intersection(&self.events);
        debug_assert!(overlapping_events.clone().collect::<Vec<_>>().len() <= 1);

        // if one event is activated, create event
        if let Some(flow_event_id) = overlapping_events.next() {
            // get event's name
            let event_name = self.symbol_table.get_name(*flow_event_id);

            // if activated, create event
            self.events.insert(id_pattern);

            // add event creation in instruction
            self.push_instr(FlowInstruction::def_let(
                flow_name,
                Expression::ident(event_name),
            ));
        }
    }

    /// Compute the instructions from a component call.
    fn handle_component_call(
        &mut self,
        pattern: &hir::Pattern,
        component_id: usize,
        inputs: &Vec<(usize, flow::Expr)>,
    ) {
        // get events that call the component
        let events = inputs.iter().filter_map(|(_, flow_expr)| {
            match flow_expr.kind {
                flow::Kind::Ident { id } => {
                    if self.events.contains(&id) {
                        let event_name = self.symbol_table.get_name(id).clone();
                        Some(Some(event_name))
                    } else if self.symbol_table.get_flow_kind(id).is_event() {
                        Some(None)
                    } else {
                        None
                    }
                }
                _ => unreachable!(), // normalized
            }
        });

        // call component with the events and update output signals
        self.call_component(component_id, pattern.clone(), events.collect());
    }

    /// Push an instruction in the current propagation branch.
    fn push_instr(&mut self, instruction: FlowInstruction) {
        self.propagations.insert(self.incoming_flow, instruction);
    }

    /// Add signal definition in current propagation branch.
    fn define_signal(&mut self, signal_id: usize, expr: Expression) {
        let signal_name = self.symbol_table.get_name(signal_id);
        self.push_instr(FlowInstruction::def_let(signal_name, expr));
        self.signals.insert(signal_id);
    }

    /// Get signal call expression.
    fn get_signal(&mut self, signal_id: usize) -> Expression {
        let signal_name = self.symbol_table.get_name(signal_id);
        // if signal not already defined, define local identifier from context value
        if !self.signals.contains(&signal_id) {
            self.push_instr(FlowInstruction::def_let(
                signal_name,
                Expression::in_ctx(signal_name),
            ));
            self.signals.insert(signal_id);
        }
        Expression::ident(signal_name)
    }

    /// Get signal name and get signal from context if needed.
    fn get_signal_name(&mut self, signal_id: usize) -> &'a String {
        let signal_name = self.symbol_table.get_name(signal_id);
        // if signal not already defined, define local identifier from context value
        if !self.signals.contains(&signal_id) {
            self.push_instr(FlowInstruction::def_let(
                signal_name,
                Expression::in_ctx(signal_name),
            ));
            self.signals.insert(signal_id);
        }
        signal_name
    }

    /// Add event definition in current propagation branch.
    fn define_event(&mut self, event_id: usize, opt_expr: Option<Expression>) {
        if let Some(expr) = opt_expr {
            let event_name = self.symbol_table.get_name(event_id);
            self.push_instr(FlowInstruction::def_let(event_name, expr));
            self.events.insert(event_id);
        }
    }

    /// Add reset timer in current propagation branch.
    fn reset_timer(&mut self, to_reset: bool, timer_id: usize, deadline: u64) {
        if to_reset {
            let timer_name = self.symbol_table.get_name(timer_id);
            self.push_instr(FlowInstruction::reset(timer_name, deadline));
        }
    }

    /// Get event call expression.
    fn get_event(&mut self, event_id: usize) -> Option<Expression> {
        // return expression only if event is defined
        if self.events.contains(&event_id) {
            let event_name = self.symbol_table.get_name(event_id);
            Some(Expression::ident(event_name))
        } else {
            None
        }
    }

    /// Add component call in current propagation branch with outputs update.
    fn call_component(
        &mut self,
        component_id: usize,
        output_pattern: hir::Pattern,
        events: Vec<Option<String>>,
    ) {
        let component_name = self.symbol_table.get_name(component_id);
        let outputs_ids = output_pattern.identifiers();
        // call component
        self.push_instr(FlowInstruction::comp_call(
            output_pattern.lir_from_hir(self.symbol_table),
            component_name,
            events,
        ));
        // update outputs
        outputs_ids
            .into_iter()
            .for_each(|output_id| self.update_ctx(output_id))
    }

    /// Add signal send in current propagation branch.
    fn send_signal(&mut self, signal_id: usize, expr: Expression) {
        let signal_name = self.symbol_table.get_name(signal_id);
        self.push_instr(FlowInstruction::send(signal_name, expr));
    }

    /// Add event send in current propagation branch.
    fn send_event(&mut self, event_id: usize, opt_expr: Option<Expression>) {
        if let Some(expr) = opt_expr {
            let event_name = self.symbol_table.get_name(event_id);
            self.push_instr(FlowInstruction::send(event_name, expr));
        }
    }

    /// Add flow send in current propagation branch.
    fn send(&mut self, flow_id: usize) {
        // insert instruction only if source is a signal or an activated event
        if self.symbol_table.get_flow_kind(flow_id).is_signal() {
            let expr = self.get_signal(flow_id);
            self.send_signal(flow_id, expr);
        } else {
            let expr = self.get_event(flow_id);
            self.send_event(flow_id, expr);
        }
    }

    /// Add context update in current propagation branch.
    fn update_ctx(&mut self, flow_id: usize) {
        // if flow is in context, add context_update instruction
        if self
            .flows_context
            .contains_element(self.symbol_table.get_name(flow_id))
        {
            let flow_name = self.symbol_table.get_name(flow_id);
            self.push_instr(FlowInstruction::update_ctx(
                flow_name,
                Expression::ident(flow_name),
            ))
        }
    }
}
