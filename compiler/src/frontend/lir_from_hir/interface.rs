prelude! {
    hir::{
        flow, interface::{
            FlowDeclaration, FlowExport, FlowImport, FlowInstantiation,
            FlowStatement, Interface, Service,
        },
    },
    lir::item::execution_machine::{
        flows_context::FlowsContext,
        service_handler::ServiceHandler,
        runtime_loop::{InputHandler, RuntimeLoop},
        ExecutionMachine,TimingEvent, InterfaceFlow,
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
        let mut input_handlers = HashMap::new();
        services_handlers.iter().for_each(|service_handler| {
            service_handler
                .flows_handling
                .iter()
                .for_each(|flow_handler| {
                    input_handlers
                        .entry(&flow_handler.arriving_flow)
                        .or_insert_with(|| vec![])
                        .push(service_handler.service.clone())
                })
        });
        let input_flows = imports
            .into_values()
            .filter_map(|import| import.lir_from_hir(symbol_table))
            .collect();
        let output_flows = exports
            .into_values()
            .map(|export| export.lir_from_hir(symbol_table))
            .collect();

        let runtime_loop = RuntimeLoop {
            input_handlers: input_handlers
                .into_iter()
                .map(|(ref_to, services)| InputHandler {
                    arriving_flow: ref_to.clone(),
                    services,
                })
                .collect(),
        };

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

        if flow_type.eq(&Typ::event(Typ::unit())) {
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
        self.into(imports, exports, timing_events, symbol_table)
    }

    fn get_flows_context(&self, symbol_table: &SymbolTable) -> FlowsContext {
        let mut flows_context = FlowsContext {
            elements: Default::default(),
        };
        self.statements
            .values()
            .for_each(|statement| statement.add_flows_context(&mut flows_context, symbol_table));
        flows_context
    }

    fn into(
        mut self,
        imports: &mut HashMap<usize, FlowImport>,
        exports: &HashMap<usize, FlowExport>,
        timing_events: &mut Vec<TimingEvent>,
        symbol_table: &mut SymbolTable,
    ) -> ServiceHandler {
        let flows_context = self.get_flows_context(symbol_table);
        symbol_table.local();
        let ctxt: flow_instr::Builder<'_> = flow_instr::Builder::new(
            &mut self,
            symbol_table,
            flows_context,
            imports,
            exports,
            timing_events,
        );
        let service_handler = service_handler::build(ctxt);
        symbol_table.global();
        service_handler
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
                    let ty = Typ::sm_event(symbol_table.get_type(id).clone());
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
                flow::Kind::ComponentCall { inputs, .. } => {
                    // get outputs' ids
                    let outputs_ids = pattern.identifiers();

                    // store output signals in flows_context
                    for output_id in outputs_ids.iter() {
                        let output_name = symbol_table.get_name(*output_id);
                        let output_type = symbol_table.get_type(*output_id);
                        flows_context.add_element(output_name.clone(), output_type)
                    }

                    inputs.iter().for_each(|(_, flow_expression)| {
                        match &flow_expression.kind {
                            // get the id of flow_expression (and check it is an idnetifier, from normalization)
                            flow::Kind::Ident { id: flow_id } => {
                                let flow_name = symbol_table.get_name(*flow_id).clone();
                                let ty = symbol_table.get_type(*flow_id);
                                if !ty.is_event() {
                                    // push in context
                                    flows_context.add_element(flow_name, ty);
                                }
                            }
                            _ => unreachable!(),
                        }
                    });
                }
                flow::Kind::Ident { .. }
                | flow::Kind::OnChange { .. }
                | flow::Kind::Timeout { .. }
                | flow::Kind::Merge { .. } => (),
            },
        }
    }
}

mod isles {
    prelude! { hir::{ Service, flow, interface::{ FlowImport, FlowStatement } } }

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
        pub fn new(
            syms: &'a SymbolTable,
            service: &'a Service,
            imports: &HashMap<usize, FlowImport>,
        ) -> Self {
            let event_to_stmts = Self::build_event_to_stmts(syms, service, imports);
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
            service: &Service,
            imports: &HashMap<usize, FlowImport>,
        ) -> HashMap<usize, Vec<usize>> {
            let mut map = HashMap::with_capacity(10);
            for (stmt_id, stmt) in service.statements.iter() {
                let mut triggered_by = |event: usize| {
                    let vec = map.entry(event).or_insert_with(Vec::new);
                    debug_assert!(!vec.contains(stmt_id));
                    vec.push(*stmt_id);
                };

                if let Some((_, inputs)) = stmt.try_get_call() {
                    // scan incoming stmt for timers
                    for import_id in service.get_dependencies(*stmt_id) {
                        if let Some(FlowImport { id: timer, .. }) = &imports.get(&import_id) {
                            if syms.is_timer(*timer) {
                                // register `stmt_id` as triggered by `input`
                                triggered_by(*timer);
                            }
                        }
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
            debug_assert! { map.iter().all(|(_, vec)| !vec.is_empty()) }
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

                // insert incoming stmt
                for stmt_id in self.service.get_dependencies(stmt_id) {
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
}

mod triggered {
    use itertools::Itertools;

    prelude! {
        graph::{DiGraphMap, DfsEvent::*}, HashSet,
        hir::{ Service, interface::{ FlowImport, FlowStatement } },
    }

    use super::isles;

    /// Graph of triggers.
    pub trait TriggersGraph<'a> {
        fn new(
            syms: &'a SymbolTable,
            service: &'a Service,
            imports: &'a HashMap<usize, FlowImport>,
        ) -> Self;
        fn get_triggered(&self, parent: usize) -> Vec<usize>;
        fn subgraph(&self, starts: impl Iterator<Item = usize>) -> DiGraphMap<usize, ()>;
        fn graph(&self) -> DiGraphMap<usize, ()>;
    }

    /// Enumerate all the implementations of TriggersGraph.
    pub enum Graph<'a> {
        EventIsles(EventIslesGraph<'a>),
        OnChange(OnChangeGraph<'a>),
    }
    impl<'a> TriggersGraph<'a> for Graph<'a> {
        fn new(
            syms: &'a SymbolTable,
            service: &'a Service,
            imports: &'a HashMap<usize, FlowImport>,
        ) -> Self {
            match conf::propag() {
                conf::PropagOption::EventIsles => {
                    Graph::EventIsles(EventIslesGraph::new(syms, service, imports))
                }
                conf::PropagOption::OnChange => {
                    Graph::OnChange(OnChangeGraph::new(syms, service, imports))
                }
            }
        }

        fn get_triggered(&self, parent: usize) -> Vec<usize> {
            match self {
                Graph::EventIsles(graph) => graph.get_triggered(parent),
                Graph::OnChange(graph) => graph.get_triggered(parent),
            }
        }

        fn subgraph(&self, starts: impl Iterator<Item = usize>) -> DiGraphMap<usize, ()> {
            match self {
                Graph::EventIsles(graph) => graph.subgraph(starts),
                Graph::OnChange(graph) => graph.subgraph(starts),
            }
        }

        fn graph(&self) -> DiGraphMap<usize, ()> {
            match self {
                Graph::EventIsles(graph) => graph.graph(),
                Graph::OnChange(graph) => graph.graph(),
            }
        }
    }

    /// Isles of statements triggered by events only.
    pub struct EventIslesGraph<'a> {
        graph: &'a DiGraphMap<usize, ()>,
        stmts: &'a HashMap<usize, FlowStatement>,
        imports: &'a HashMap<usize, FlowImport>,
        isles: isles::Isles,
    }
    impl<'a> EventIslesGraph<'a> {
        /// Returns the identifiers of flows that are defined by the statement.
        fn get_def_flows(&self, id: usize) -> Vec<usize> {
            if let Some(stmt) = self.stmts.get(&id) {
                stmt.get_identifiers()
            } else if let Some(import) = self.imports.get(&id) {
                vec![import.id]
            } else {
                vec![]
            }
        }
        /// Tells if the statements is a component call.
        fn is_comp_call(&self, id: usize) -> bool {
            self.stmts
                .get(&id)
                .map_or(false, FlowStatement::is_comp_call)
        }
    }
    impl<'a> TriggersGraph<'a> for EventIslesGraph<'a> {
        fn new(
            syms: &'a SymbolTable,
            service: &'a Service,
            imports: &'a HashMap<usize, FlowImport>,
        ) -> Self {
            // create events isles
            let mut isle_builder = isles::IsleBuilder::new(syms, service, &imports);
            isle_builder.trace_events(service.get_flows_ids(imports.values()));
            let isles = isle_builder.into_isles();

            EventIslesGraph {
                graph: &service.graph,
                stmts: &service.statements,
                imports,
                isles,
            }
        }

        fn get_triggered(&self, parent: usize) -> Vec<usize> {
            // get graph dependencies
            let dependencies = self.graph.neighbors(parent).filter_map(|child| {
                // filter component call because they will appear in isles
                if self.is_comp_call(child) {
                    return None;
                }
                Some(child)
            });

            // get isles dependencies
            let isles = self
                .get_def_flows(parent)
                .into_iter()
                .filter_map(|parent_flow| self.isles.get_isle_for(parent_flow))
                .flatten()
                .map(|to_insert| *to_insert);

            // extend stack with union of event isle and dependencies
            isles.chain(dependencies).unique().collect()
        }

        fn subgraph(&self, starts: impl Iterator<Item = usize>) -> DiGraphMap<usize, ()> {
            let mut trig_graph = DiGraphMap::new();
            // init stack and seen set
            let (mut stack, mut seen) = (vec![], HashSet::new());
            starts.for_each(|id| {
                trig_graph.add_node(id);
                stack.push(id);
                seen.insert(id);
            });
            // loop on stack
            while let Some(parent) = stack.pop() {
                let neighbors = self.get_triggered(parent);
                for child in neighbors {
                    // add in subgraph of triggers
                    trig_graph.add_edge(parent, child, ());
                    // only insert in stack if not seen
                    if seen.insert(child) {
                        stack.push(child);
                    }
                }
            }
            trig_graph
        }

        fn graph(&self) -> DiGraphMap<usize, ()> {
            self.subgraph(self.graph.nodes())
        }
    }

    /// Statements triggered by all changes.
    pub struct OnChangeGraph<'a> {
        graph: &'a DiGraphMap<usize, ()>,
    }
    impl<'a> TriggersGraph<'a> for OnChangeGraph<'a> {
        fn new(
            _syms: &'a SymbolTable,
            service: &'a Service,
            _imports: &'a HashMap<usize, FlowImport>,
        ) -> Self {
            OnChangeGraph {
                graph: &service.graph,
            }
        }

        fn get_triggered(&self, parent: usize) -> Vec<usize> {
            // get graph dependencies
            self.graph.neighbors(parent).collect()
        }

        fn subgraph(&self, starts: impl Iterator<Item = usize>) -> DiGraphMap<usize, ()> {
            let mut trig_graph = DiGraphMap::new();
            let starts = starts.collect::<Vec<_>>();
            starts.iter().for_each(|id| {
                trig_graph.add_node(*id);
            });
            petgraph::visit::depth_first_search(&self.graph, starts, |event| match event {
                CrossForwardEdge(parent, child)
                | BackEdge(parent, child)
                | TreeEdge(parent, child) => {
                    // add in subgraph of triggers
                    trig_graph.add_edge(parent, child, ());
                }
                Discover(_, _) | Finish(_, _) => {}
            });
            trig_graph
        }

        fn graph(&self) -> DiGraphMap<usize, ()> {
            self.graph.clone()
        }
    }
}

mod service_handler {
    prelude! {
        lir::{
            item::execution_machine::{
                service_handler::{FlowHandler, FlowInstruction, MatchArm, ServiceHandler},
                ArrivingFlow,
            },
            Pattern,
        },
        synced::{Builder, Synced},
    }

    use frontend::lir_from_hir::interface::clean_synced;

    use super::{flow_instr, from_synced, triggered::TriggersGraph};

    /// Compute the instruction propagating the changes of the input flow.
    fn propagate<'a>(
        ctxt: &mut flow_instr::Builder<'a>,
        stmt_id: usize,
        flow_id: usize,
    ) -> FlowInstruction {
        if ctxt.syms().is_delay(flow_id) {
            propagate_input_store(ctxt, flow_id)
        } else {
            ctxt.set_multiple_inputs(false);
            flow_instruction(ctxt, std::iter::once(stmt_id))
        }
    }

    /// Compute the instruction propagating the changes of the input store.
    fn propagate_input_store<'a>(
        ctxt: &mut flow_instr::Builder<'a>,
        delay_id: usize,
    ) -> FlowInstruction {
        debug_assert!(ctxt.is_clear());
        debug_assert!(ctxt.syms().is_delay(delay_id));
        let syms = ctxt.syms();

        // this is an ORDERED list of the input flows
        let inputs = ctxt.inputs().collect::<Vec<_>>();
        let flows_names = inputs
            .iter()
            .map(|(_, import_id)| syms.get_name(*import_id).clone());

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
                        let flow_name = syms.get_name(*import_id);
                        if syms.is_timer(*import_id) {
                            Pattern::some(Pattern::tuple(vec![
                                Pattern::literal(Constant::unit(Default::default())),
                                Pattern::ident(format!("{}_instant", flow_name)),
                            ]))
                        } else {
                            Pattern::some(Pattern::tuple(vec![
                                Pattern::ident(flow_name),
                                Pattern::ident(format!("{}_instant", flow_name)),
                            ]))
                        }
                    } else {
                        Pattern::none()
                    }
                })
                .collect();
            // compute the instruction that will propagate changes
            ctxt.set_multiple_inputs(true);
            let instr = flow_instruction(ctxt, imports.into_iter());
            MatchArm::new(patterns, instr)
        });

        FlowInstruction::handle_delay(flows_names, arms)
    }

    /// Compute the instruction propagating the changes of the input flows.
    fn flow_instruction<'a>(
        ctxt: &mut flow_instr::Builder<'a>,
        imports: impl Iterator<Item = usize>,
    ) -> FlowInstruction {
        debug_assert!(ctxt.is_clear());
        // construct subgraph representing the propagation of 'imports'
        let subgraph = &ctxt.graph().subgraph(imports);

        let synced = if conf::para() {
            // if config is 'para' then build 'synced' with //-algo
            if subgraph.node_count() == 0 {
                return FlowInstruction::seq(vec![]);
            }
            let builder = Builder::<flow_instr::Builder, ()>::new(subgraph);
            builder.run(ctxt).expect("oh no")
        } else {
            // else, construct an ordered sequence of the instrs
            let ord_instrs = petgraph::algo::toposort(&subgraph, None).expect("no cycle expected");
            let seq: Vec<_> = ord_instrs
                .into_iter()
                .map(|i| Synced::instr(i, ctxt))
                .collect();
            if seq.is_empty() {
                return FlowInstruction::seq(vec![]);
            }
            Synced::seq(seq, ctxt)
        };

        // puts the exports out of parallel instructions
        let synced = clean_synced::run(ctxt, synced);
        // produce the corresponding LIR instruction
        let instr = from_synced::run(ctxt, synced);

        ctxt.clear();
        instr
    }

    /// Compute the input flow's handler.
    fn flow_handler<'a>(
        ctxt: &mut flow_instr::Builder<'a>,
        stmt_id: usize,
        flow_id: usize,
    ) -> FlowHandler {
        // construct the instruction to perform
        let instruction = propagate(ctxt, stmt_id, flow_id);

        let flow_name = ctxt.syms().get_name(flow_id).clone();
        // determine weither this arriving flow is a timing event
        let arriving_flow = if ctxt.syms().is_delay(flow_id) {
            ArrivingFlow::ServiceDelay(flow_name)
        } else if ctxt.syms().is_period(flow_id) {
            ArrivingFlow::Period(flow_name)
        } else if ctxt.syms().is_deadline(flow_id) {
            ArrivingFlow::Deadline(flow_name)
        } else if ctxt.syms().is_timeout(flow_id) {
            ArrivingFlow::ServiceTimeout(flow_name)
        } else {
            let flow_type = ctxt.syms().get_type(flow_id).clone();
            let path = ctxt.syms().get_path(flow_id).clone();
            ArrivingFlow::Channel(flow_name, flow_type, path)
        };

        FlowHandler {
            arriving_flow,
            instruction,
        }
    }

    /// Compute the service handler.
    pub fn build<'a>(mut ctxt: flow_instr::Builder<'a>) -> ServiceHandler {
        // get service's name
        let service = ctxt.service_name();
        // create flow handlers according to propagations of every incomming flows
        let flows_handling: Vec<_> = ctxt
            .service_imports()
            .map(|(stmt_id, import_id)| flow_handler(&mut ctxt, stmt_id, import_id))
            .collect();
        // destroy 'ctxt'
        let (flows_context, components) = ctxt.destroy();

        ServiceHandler {
            service,
            components,
            flows_handling,
            flows_context,
        }
    }
}

mod flow_instr {
    prelude! {
        quote::format_ident,
        graph::{DfsEvent::*, DiGraphMap},
        hir::{
            flow,
            interface::{
                FlowDeclaration, FlowInstantiation,
                FlowStatement, FlowImport, FlowExport,
            },
            IdentifierCreator, Service,
        },
        lir::item::execution_machine::{
            flows_context::FlowsContext, service_handler::{Expression, FlowInstruction},
            TimingEvent, TimingEventKind,
        },
    }

    use super::{
        triggered::{self, TriggersGraph},
        LIRFromHIR,
    };

    /// A context to build [FlowInstruction]s.
    pub struct Builder<'a> {
        /// Context of the service.
        flows_context: FlowsContext,
        /// Symbol table.
        syms: &'a SymbolTable,
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
        components: Vec<String>,
        /// Triggers graph,
        graph: triggered::Graph<'a>,
    }

    impl<'a> Builder<'a> {
        /// Create a Builder.
        ///
        /// After creating the builder, you only need to [propagate](Self::propagate) the input flows.
        /// This will create the instructions to run when the input flow arrives.
        pub fn new(
            service: &'a mut Service,
            syms: &'a mut SymbolTable,
            mut flows_context: FlowsContext,
            imports: &'a mut HashMap<usize, FlowImport>,
            exports: &'a HashMap<usize, FlowExport>,
            timing_events: &'a mut Vec<TimingEvent>,
        ) -> Self {
            let mut identifier_creator = IdentifierCreator::from(
                service.get_flows_names(syms).chain(
                    imports
                        .values()
                        .map(|import| syms.get_name(import.id).clone()),
                ),
            );
            let mut components = vec![];
            // retrieve timer and onchange events from service
            let (stmts_timers, on_change_events) = Self::build_stmt_events(
                &mut identifier_creator,
                service,
                syms,
                &mut flows_context,
                imports,
                timing_events,
                &mut components,
            );
            // add events related to service's constrains
            Self::build_constrains_events(
                &mut identifier_creator,
                service,
                syms,
                imports,
                timing_events,
            );

            // create triggered graph
            let graph = triggered::Graph::new(syms, service, imports);
            // construct [stmt -> imports]
            let stmts_imports = Self::build_stmts_imports(&graph.graph(), imports);

            Builder {
                flows_context,
                syms,
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
        pub fn syms(&self) -> &'a SymbolTable {
            self.syms
        }
        pub fn graph(&self) -> &triggered::Graph<'a> {
            &self.graph
        }
        pub fn service_imports(&self) -> impl Iterator<Item = (usize, usize)> + 'a {
            self.imports
                .iter()
                .filter(|(stmt_id, import)| {
                    // 'service_delay' is not in the graph
                    // (it does not trigger instructions but the propagation of the 'input_store')
                    self.syms.is_service_delay(self.service.id, import.id)
                        || self.service.graph.edges(**stmt_id).next().is_some()
                })
                .map(|(stmt_id, import)| (*stmt_id, import.id))
        }
        pub fn service_name(&self) -> String {
            self.syms.get_name(self.service.id).to_string()
        }
        pub fn inputs(&self) -> impl Iterator<Item = (usize, usize)> + 'a {
            self.service_imports().filter(|(_, import_id)| {
                !(self.syms.is_delay(*import_id) || self.syms.is_timeout(*import_id))
            })
        }
        pub fn set_multiple_inputs(&mut self, multiple_inputs: bool) {
            self.multiple_inputs = multiple_inputs
        }
        pub fn destroy(self) -> (FlowsContext, Vec<String>) {
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
            for (stmt_id, import) in imports.iter() {
                petgraph::visit::depth_first_search(graph, std::iter::once(*stmt_id), |event| {
                    match event {
                        CrossForwardEdge(_, child) | BackEdge(_, child) | TreeEdge(_, child) => {
                            stmts_imports
                                .entry(child)
                                .or_insert(vec![])
                                .push((*stmt_id, import.id))
                        }
                        Discover(_, _) | Finish(_, _) => {}
                    }
                });
            }
            stmts_imports
        }

        /// Adds events related to statements.
        fn build_stmt_events(
            identifier_creator: &mut IdentifierCreator,
            service: &mut Service,
            syms: &mut SymbolTable,
            flows_context: &mut FlowsContext,
            imports: &mut HashMap<usize, FlowImport>,
            timing_events: &mut Vec<TimingEvent>,
            components: &mut Vec<String>,
        ) -> (HashMap<usize, usize>, HashMap<usize, usize>) {
            // collects components, timing events, on_change_events that are present in the service
            let mut stmts_timers = HashMap::new();
            let mut on_change_events = HashMap::new();
            service.statements.iter().for_each(|(stmt_id, statement)| {
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
                                let event_name = syms.get_name(flow_event_id).clone();

                                // add new event into the identifier creator
                                let fresh_name =
                                    identifier_creator.new_identifier_with("", &event_name, "old");
                                let typing = syms.get_type(flow_event_id).clone();
                                let kind = syms.get_flow_kind(flow_event_id).clone();
                                let fresh_id = syms.insert_fresh_flow(
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
                                let flow_name = syms.get_name(pattern.identifiers().pop().unwrap());
                                let fresh_name =
                                    identifier_creator.fresh_identifier("period", flow_name);
                                let typing = Typ::event(Typ::unit());
                                let fresh_id =
                                    syms.insert_fresh_period(fresh_name.clone(), *period_ms);

                                // add timing_event in imports
                                let fresh_statement_id = syms.get_fresh_id();
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
                                let flow_name = syms.get_name(pattern.identifiers().pop().unwrap());
                                let fresh_name =
                                    identifier_creator.fresh_identifier("timeout", flow_name);
                                let typing = Typ::event(Typ::unit());
                                let fresh_id =
                                    syms.insert_fresh_deadline(fresh_name.clone(), *deadline);

                                // add timing_event in imports
                                let fresh_statement_id = syms.get_fresh_id();
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
                                let comp_name = syms.get_name(*component_id).clone();
                                // add potential period constrains
                                if let Some(period) = syms.get_node_period(*component_id) {
                                    // add new timing event into the identifier creator
                                    let fresh_name =
                                        identifier_creator.fresh_identifier("period", &comp_name);
                                    let typing = Typ::event(Typ::unit());
                                    let fresh_id =
                                        syms.insert_fresh_period(fresh_name.clone(), period);
                                    syms.set_node_period_id(*component_id, fresh_id);

                                    // add timing_event in imports
                                    let fresh_statement_id = syms.get_fresh_id();
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
                                        kind: TimingEventKind::Period(period.clone()),
                                    })
                                }
                                components.push(comp_name)
                            }
                        }
                    }
                };
            });

            (stmts_timers, on_change_events)
        }

        /// Adds events related to service's constrains.
        fn build_constrains_events(
            identifier_creator: &mut IdentifierCreator,
            service: &mut Service,
            syms: &mut SymbolTable,
            imports: &mut HashMap<usize, FlowImport>,
            timing_events: &mut Vec<TimingEvent>,
        ) {
            let min_delay = service.constrains.0;
            // add new timing event into the identifier creator
            let fresh_name =
                identifier_creator.fresh_identifier("delay", syms.get_name(service.id));
            let typing = Typ::event(Typ::unit());
            let fresh_id = syms.insert_service_delay(fresh_name.clone(), service.id, min_delay);
            // add timing_event in imports
            let fresh_statement_id = syms.get_fresh_id();
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

            let max_timeout = service.constrains.1;
            // add new timing event into the identifier creator
            let fresh_name =
                identifier_creator.fresh_identifier("timeout", syms.get_name(service.id));
            let typing = Typ::event(Typ::unit());
            let fresh_id = syms.insert_service_timeout(fresh_name.clone(), service.id, max_timeout);
            // add timing_event in imports
            let fresh_statement_id = syms.get_fresh_id();
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
            service.statements.keys().for_each(|stmt_id| {
                if service.statements[stmt_id].is_comp_call() {
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
                .filter(|event_id| !self.syms.is_timer(**event_id))
                .map(|event_id| FlowInstruction::init_event(self.syms.get_name(*event_id)))
        }

        /// Compute the instruction from an import.
        pub fn handle_import(&mut self, flow_id: usize) -> FlowInstruction {
            if self.syms.get_flow_kind(flow_id).is_event() {
                // add to events set
                self.events.insert(flow_id);
                if !self.syms.is_timer(flow_id) {
                    // store the event in the local reference
                    let event_name = self.syms.get_name(flow_id);
                    let expr = Expression::some(Expression::ident(event_name));
                    self.define_event(flow_id, expr)
                } else if self.syms.is_period(flow_id) {
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
            pattern: &hir::Pattern,
            flow_expression: &flow::Expr,
        ) -> FlowInstruction {
            let dependencies = flow_expression.get_dependencies();
            match &flow_expression.kind {
                flow::Kind::Ident { id } => self.handle_ident(pattern, *id),
                flow::Kind::Sample { .. } => self.handle_sample(stmt_id, pattern, dependencies),
                flow::Kind::Scan { .. } => self.handle_scan(stmt_id, pattern, dependencies),
                flow::Kind::Timeout { .. } => self.handle_timeout(stmt_id, pattern, dependencies),
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

        /// Compute the instruction from an identifier expression.
        fn handle_ident(&mut self, pattern: &hir::Pattern, id_source: usize) -> FlowInstruction {
            // get the id of pattern's flow, debug-check there is only one flow
            let mut ids = pattern.identifiers();
            debug_assert!(ids.len() == 1);
            let id_pattern = ids.pop().unwrap();

            // insert instruction only if source is a signal or an activated event
            let def = if self.syms.get_flow_kind(id_source).is_signal() {
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
            pattern: &hir::Pattern,
            mut dependencies: Vec<usize>,
        ) -> FlowInstruction {
            // get the id of pattern's flow, debug-check there is only one flow
            let mut ids = pattern.identifiers();
            debug_assert!(ids.len() == 1);
            let id_pattern = ids.pop().unwrap();
            let flow_name = self.syms.get_name(id_pattern);

            // get the source id, debug-check there is only one flow
            debug_assert!(dependencies.len() == 1);
            let id_source = dependencies.pop().unwrap();
            let source_name = self.syms.get_name(id_source);

            let timer_id = self.stmts_timers[&stmt_id];

            let mut instrs = vec![];
            // source is an event, look if it is defined
            if self.events.contains(&id_source) {
                // if activated, store event value in context
                let update =
                    FlowInstruction::update_ctx(source_name, Expression::event(source_name));
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
                let take_update =
                    FlowInstruction::update_ctx(flow_name, Expression::take_from_ctx(source_name));
                instrs.push(take_update)
            }

            FlowInstruction::seq(instrs)
        }

        /// Compute the instruction from a scan expression.
        fn handle_scan(
            &mut self,
            stmt_id: usize,
            pattern: &hir::Pattern,
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
            pattern: &hir::Pattern,
            mut dependencies: Vec<usize>,
        ) -> FlowInstruction {
            // get the id of pattern's flow, debug-check there is only one flow
            let mut ids = pattern.identifiers();
            debug_assert!(ids.len() == 1);
            let id_pattern = ids.pop().unwrap();

            // get the source id, debug-check there is only one flow
            debug_assert!(dependencies.len() == 1);
            let id_source = dependencies.pop().unwrap();
            let source_name = self.syms.get_name(id_source);

            let timer_id = self.stmts_timers[&stmt_id].clone();

            let occurences = (
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
            match occurences {
                (true, true) => source_instr(Some(timer_instr())),
                (true, false) => source_instr(None),
                (false, true) => timer_instr(),
                (false, false) => {
                    unreachable!("'timeout' should be activated by either its source or its timer")
                }
            }
        }

        /// Compute the instruction from a throttle expression.
        fn handle_throttle(
            &self,
            pattern: &hir::Pattern,
            mut dependencies: Vec<usize>,
            delta: Constant,
        ) -> FlowInstruction {
            // get the id of pattern's flow, debug-check there is only one flow
            let mut ids = pattern.identifiers();
            debug_assert!(ids.len() == 1);
            let id_pattern = ids.pop().unwrap();
            let flow_name = self.syms.get_name(id_pattern);

            // get the source id, debug-check there is only one flow
            debug_assert!(dependencies.len() == 1);
            let id_source = dependencies.pop().unwrap();
            let source_name = self.syms.get_name(id_source);

            // update created signal
            let expr = self.get_signal(id_source);
            FlowInstruction::if_throttle(
                flow_name,
                source_name,
                delta,
                FlowInstruction::update_ctx(flow_name, expr),
            )
        }

        /// Compute the instruction from an on_change expression.
        fn handle_on_change(
            &mut self,
            pattern: &hir::Pattern,
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
            let old_event_name = self.syms.get_name(id_old_event);

            // detect changes on signal
            let expr = Expression::some(self.get_signal(id_source));
            let event_def = self.define_event(id_pattern, expr);
            let then = vec![
                FlowInstruction::update_ctx(old_event_name.clone(), self.get_signal(id_source)),
                event_def,
            ];
            FlowInstruction::if_change(
                old_event_name,
                self.get_signal(id_source),
                FlowInstruction::seq(then),
            )
        }

        /// Compute the instruction from a merge expression.
        fn handle_merge(
            &mut self,
            pattern: &hir::Pattern,
            mut dependencies: Vec<usize>,
        ) -> FlowInstruction {
            // get the id of pattern's flow, debug-check there is only one flow
            let mut ids = pattern.identifiers();
            debug_assert!(ids.len() == 1);
            let id_pattern = ids.pop().unwrap();

            // get the source id, debug-check there is only one flow
            debug_assert!(dependencies.len() == 2);
            let id_source_1 = dependencies.pop().unwrap();
            let event_1 = self.syms.get_name(id_source_1).clone();
            let id_source_2 = dependencies.pop().unwrap();
            let event_2 = self.syms.get_name(id_source_2).clone();

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
                (false, false) => unreachable!("'merge' should be activated by one of its sources"),
            }
        }

        /// Compute the instruction from a component call.
        fn handle_component_call(
            &mut self,
            pattern: &hir::Pattern,
            component_id: usize,
            inputs: &Vec<(usize, flow::Expr)>,
        ) -> FlowInstruction {
            // get events that might call the component
            let (mut signals, mut events) = (vec![], vec![]);
            inputs.iter().for_each(|(input_id, flow_expr)| {
                match flow_expr.kind {
                    flow::Kind::Ident { id } => {
                        let input_name = self.syms.get_name(*input_id).clone();
                        if self.syms.get_flow_kind(id).is_event() {
                            if self.events.contains(&id) {
                                let event_name = self.syms.get_name(id).clone();
                                events.push((input_name, Some(event_name)));
                            } else {
                                events.push((input_name, None));
                            }
                        } else {
                            let signal_name = self.syms.get_name(id).clone();
                            signals.push((input_name, signal_name));
                        }
                    }
                    _ => unreachable!(), // normalized
                }
            });

            // call component with the events and update output signals
            self.call_component(component_id, pattern.clone(), signals, events)
        }

        /// Add signal definition in current propagation branch.
        fn define_signal(&mut self, signal_id: usize, expr: Expression) -> FlowInstruction {
            let signal_name = self.syms.get_name(signal_id);
            self.signals.insert(signal_id);
            FlowInstruction::def_let(signal_name, expr)
        }

        /// Get signal call expression.
        fn get_signal(&self, signal_id: usize) -> Expression {
            let signal_name = self.syms.get_name(signal_id);
            // if signal not already defined, get from context value
            if !self.signals.contains(&signal_id) {
                Expression::in_ctx(signal_name)
            } else {
                Expression::ident(signal_name)
            }
        }

        /// Add event definition in current propagation branch.
        fn define_event(&mut self, event_id: usize, expr: Expression) -> FlowInstruction {
            let event_name = self.syms.get_name(event_id);
            self.events.insert(event_id);
            FlowInstruction::update_event(event_name, expr)
        }

        /// Add reset timer in current propagation branch.
        fn reset_timer(&self, timer_id: usize, import_flow: usize) -> FlowInstruction {
            let timer_name = self.syms.get_name(timer_id);
            let import_name = self.syms.get_name(import_flow);
            FlowInstruction::reset(timer_name, import_name)
        }

        /// Get event call expression.
        fn get_event(&self, event_id: usize) -> Expression {
            let event_name = self.syms.get_name(event_id);
            Expression::event(event_name)
        }

        /// Add component call in current propagation branch with outputs update.
        fn call_component(
            &mut self,
            component_id: usize,
            output_pattern: hir::Pattern,
            signals: Vec<(String, String)>,
            events: Vec<(String, Option<String>)>,
        ) -> FlowInstruction {
            let component_name = self.syms.get_name(component_id);
            let outputs_ids = output_pattern.identifiers();

            // call component
            let mut instrs = vec![FlowInstruction::comp_call(
                output_pattern.lir_from_hir(self.syms),
                component_name,
                signals.clone(),
                events.clone(),
            )];
            // update outputs: context signals and all events
            let updates = outputs_ids.into_iter().filter_map(|output_id| {
                if self.syms.get_flow_kind(output_id).is_event() {
                    let expr = self.get_event(output_id);
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

            // call component when activated by its period
            if let Some(period_id) = self.syms.get_node_period_id(component_id) {
                if self.events.contains(&period_id) {
                    return comp_call;
                }
            }
            // call component when activated by inputs
            let events: Vec<String> = events
                .into_iter()
                .filter_map(|(_, opt_event)| opt_event)
                .collect();
            let signals = match conf::propag() {
                conf::PropagOption::EventIsles => vec![], // isles activated on events
                conf::PropagOption::OnChange => {
                    signals.into_iter().map(|(_, signal)| signal).collect()
                }
            };
            FlowInstruction::if_activated(events, signals, comp_call, None)
        }

        /// Add signal send in current propagation branch.
        fn send_signal(&self, signal_id: usize, import_flow: usize) -> FlowInstruction {
            let signal_name = self.syms.get_name(signal_id);
            let expr = self.get_signal(signal_id);
            if self.multiple_inputs {
                FlowInstruction::send(signal_name, expr, false)
            } else {
                let import_name = self.syms.get_name(import_flow);
                FlowInstruction::send_from(signal_name, expr, import_name, false)
            }
        }

        /// Add event send in current propagation branch.
        fn send_event(&self, event_id: usize, import_flow: usize) -> FlowInstruction {
            let event_name = self.syms.get_name(event_id);
            let expr = Expression::ident(event_name);
            if self.multiple_inputs {
                FlowInstruction::send(event_name, expr, true)
            } else {
                let import_name = self.syms.get_name(import_flow);
                FlowInstruction::send_from(event_name, expr, import_name, true)
            }
        }

        /// Add flow send in current propagation branch.
        pub fn send(&self, stmt_id: usize, flow_id: usize) -> FlowInstruction {
            let import_flow = self.get_stmt_import(stmt_id);
            // insert instruction only if source is a signal or an activated event
            if self.syms.get_flow_kind(flow_id).is_signal() {
                self.send_signal(flow_id, import_flow)
            } else {
                self.send_event(flow_id, import_flow)
            }
        }

        /// Add context update in current propagation branch.
        fn update_ctx(&self, flow_id: usize) -> Option<FlowInstruction> {
            // if flow is in context, add context_update instruction
            if self
                .flows_context
                .contains_element(self.syms.get_name(flow_id))
            {
                let expr: Expression = if self.syms.get_flow_kind(flow_id).is_event() {
                    self.get_event(flow_id)
                } else {
                    self.get_signal(flow_id)
                };
                let flow_name = self.syms.get_name(flow_id);
                Some(FlowInstruction::update_ctx(flow_name, expr))
            } else {
                None
            }
        }
    }
}

mod from_synced {
    prelude! { just
        BTreeMap as Map,
        hir::interface::{ FlowStatement, FlowDeclaration, FlowInstantiation },
        lir::item::execution_machine::service_handler::{ParaMethod, FlowInstruction},
        synced::{ CtxSpec, Synced },
    }

    use super::flow_instr;

    pub trait FromSynced<Ctx: CtxSpec + ?Sized>: Sized {
        fn prefix(ctxt: &mut Ctx) -> Self;
        fn suffix(ctxt: &mut Ctx) -> Self;
        fn from_instr(ctxt: &mut Ctx, instr: Ctx::Instr) -> Self;
        fn from_seq(ctxt: &mut Ctx, seq: Vec<Self>) -> Self;
        fn from_para(ctxt: &mut Ctx, para: Map<ParaMethod, Vec<Self>>) -> Self;
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

    pub fn run<Ctx, Instr>(ctxt: &mut Ctx, synced: Synced<Ctx>) -> Instr
    where
        Ctx: CtxSpec + ?Sized,
        Ctx::Cost: IntoParaMethod,
        Instr: FromSynced<Ctx>,
    {
        let mut stack: Stack<Ctx, Instr> = Stack::new();
        let mut acc = None;
        let mut curr = synced;

        'go_down: loop {
            debug_assert!(acc.is_none());
            match curr {
                Synced::Instr(instr, _) => acc = Some(Instr::from_instr(ctxt, instr)),
                Synced::Seq(mut todo, _) => {
                    todo.reverse();
                    curr = todo
                        .pop()
                        .expect("there should be a synced in this sequence");
                    stack.push(Frame::Seq { done: vec![], todo });
                    continue 'go_down;
                }
                Synced::Para(mut todo, _) => {
                    let (cost, mut cost_todo) = todo
                        .pop_first()
                        .expect("there should be synceds in this parallel execution");
                    curr = cost_todo
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
                        let prefix = Instr::prefix(ctxt);
                        let instr = acc.expect("there should be an instruction to return");
                        let suffix = Instr::suffix(ctxt);
                        return Instr::from_seq(ctxt, vec![prefix, instr, suffix]);
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
                            curr = cost_todo
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
                            acc = Some(Instr::from_para(ctxt, done));
                            continue 'go_up;
                        }
                    }
                    Some(Frame::Seq { mut done, mut todo }) => {
                        let instr = std::mem::take(&mut acc)
                            .expect("there should be an instruction to sequence");
                        done.push(instr);
                        if let Some(curr_todo) = todo.pop() {
                            curr = curr_todo;
                            stack.push(Frame::Seq { done, todo });
                            continue 'go_down;
                        } else {
                            acc = Some(Instr::from_seq(ctxt, done));
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
            ctxt: &mut flow_instr::Builder,
            instr: <flow_instr::Builder as CtxSpec>::Instr,
        ) -> Self {
            // get flow statement related to instr
            if let Some(flow_statement) = ctxt.get_stmt(instr) {
                match flow_statement.clone() {
                    FlowStatement::Declaration(FlowDeclaration {
                        pattern,
                        flow_expression,
                        ..
                    })
                    | FlowStatement::Instantiation(FlowInstantiation {
                        pattern,
                        flow_expression,
                        ..
                    }) => ctxt.handle_expr(instr, &pattern, &flow_expression),
                }
            } else
            // get flow export related to instr
            if let Some(export) = ctxt.get_export(instr) {
                ctxt.send(instr, export.id)
            } else
            // get flow import related to instr
            if let Some(import) = ctxt.get_import(instr) {
                ctxt.handle_import(import.id)
            } else {
                unreachable!()
            }
        }

        fn from_seq(_ctxt: &mut flow_instr::Builder, seq: Vec<Self>) -> Self {
            FlowInstruction::seq(seq)
        }

        fn from_para(_ctxt: &mut flow_instr::Builder, para: Map<ParaMethod, Vec<Self>>) -> Self {
            FlowInstruction::para(para)
        }

        fn prefix(ctxt: &mut flow_instr::Builder<'a>) -> Self {
            // init events that should be declared as &mut.
            let init_events = ctxt.init_events().collect::<Vec<_>>();
            FlowInstruction::seq(init_events)
        }

        fn suffix(_ctxt: &mut flow_instr::Builder<'a>) -> Self {
            FlowInstruction::seq(vec![])
        }
    }
}

mod clean_synced {
    use super::flow_instr;

    prelude! { just
        BTreeMap as Map,
        synced::{ CtxSpec, Synced },
    }

    pub trait IsExport: CtxSpec {
        fn is_export(ctxt: &Self, instr: Self::Instr) -> bool;
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
    pub fn run<Ctx>(ctxt: &Ctx, synced: Synced<Ctx>) -> Synced<Ctx>
    where
        Ctx: CtxSpec + IsExport + ?Sized,
    {
        let mut stack: Stack<Ctx> = Stack::new();
        let mut acc = None;
        let mut curr = synced;

        'go_down: loop {
            debug_assert!(acc.is_none());
            match curr {
                Synced::Instr(_, _) => acc = Some(curr),
                Synced::Seq(mut todo, _) => {
                    todo.reverse();
                    curr = todo
                        .pop()
                        .expect("there should be a synced in this sequence");
                    stack.push(Frame::Seq { done: vec![], todo });
                    continue 'go_down;
                }
                Synced::Para(_, _) => {
                    let (para, exports) = extract_exports(ctxt, curr);
                    acc = Some(Synced::seq(vec![para, exports], ctxt));
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
                        if let Some(curr_todo) = todo.pop() {
                            curr = curr_todo;
                            stack.push(Frame::Seq { done, todo });
                            continue 'go_down;
                        } else {
                            acc = Some(Synced::seq(done, ctxt));
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
    fn extract_exports<Ctx>(ctxt: &Ctx, synced: Synced<Ctx>) -> (Synced<Ctx>, Synced<Ctx>)
    where
        Ctx: CtxSpec + IsExport + ?Sized,
    {
        let mut stack: ExtractStack<Ctx> = ExtractStack::new();
        let mut acc = None;
        let mut exports = vec![];
        let mut curr = synced;

        'go_down: loop {
            debug_assert!(acc.is_none());
            match curr {
                Synced::Instr(instr, _) => {
                    if IsExport::is_export(ctxt, instr) {
                        exports.push(curr)
                    } else {
                        acc = Some(curr)
                    }
                }
                Synced::Seq(mut todo, _) => {
                    todo.reverse();
                    curr = todo
                        .pop()
                        .expect("there should be a synced in this sequence");
                    stack.push(ExtractFrame::Seq { done: vec![], todo });
                    continue 'go_down;
                }
                Synced::Para(mut todo, _) => {
                    let (cost, mut cost_todo) = todo
                        .pop_first()
                        .expect("there should be synceds in this parallel execution");
                    curr = cost_todo
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
                        let synced = acc.expect("there should be a synced to return");
                        let exports = Synced::seq(exports, ctxt);
                        return (synced, exports);
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
                            curr = cost_todo
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
                            acc = Some(Synced::para(done, ctxt));
                            continue 'go_up;
                        }
                    }
                    Some(ExtractFrame::Seq { mut done, mut todo }) => {
                        if let Some(instr) = std::mem::take(&mut acc) {
                            done.push(instr);
                        }
                        if let Some(curr_todo) = todo.pop() {
                            curr = curr_todo;
                            stack.push(ExtractFrame::Seq { done, todo });
                            continue 'go_down;
                        } else if done.is_empty() {
                            continue 'go_up;
                        } else {
                            acc = Some(Synced::seq(done, ctxt));
                            continue 'go_up;
                        }
                    }
                }
            }
        }
    }

    impl<'a> IsExport for flow_instr::Builder<'a> {
        fn is_export(ctxt: &Self, instr: Self::Instr) -> bool {
            ctxt.get_export(instr).is_some()
        }
    }
}
