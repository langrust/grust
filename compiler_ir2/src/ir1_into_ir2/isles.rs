prelude! { ir1::{ Service, flow, interface::{ FlowImport, FlowStatement } } }

/// An *"isle"* for some event `e` is all (and only) the statements to run when receiving `e`.
///
/// This structure is only meant to be used immutably *after* it is created by [`IsleBuilder`],
/// in fact all `&mut self` functions on [`Isles`] are private.
///
/// Given a service and some statements, each event triggers statements that feature call to
/// eventful component. To actually run them, we need to update their inputs, which means that
/// we need to know the event-free statements (including event-free component calls) that
/// produce the inputs for each eventful component call triggered.
///
/// The *"isle"* for event `e` is the list of statements from the service that need to run to
/// fully compute the effect of receiving `e` (including top stateful calls). The isle is a
/// sub-list of the original list of statements, in particular it is ordered the same way.
pub struct Isles {
    /// Maps event indices to component isles.
    events_to_isles: HashMap<usize, Vec<usize>>,
}
impl Isles {
    /// Constructor for an event capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events_to_isles: HashMap::with_capacity(capacity),
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
    /// Note that the statements in the isles are not (insert-)sorted by this function, that's
    /// why it is private. Each isle is populated by [`IsleBuilder::trace_event`], which does
    /// sort the isle it creates before returning.
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
    /// The `bool` flag indicates the statement is at top-level, meaning it should be inserted
    /// in the isle despite being stateful.
    stack: Vec<(usize, bool)>,
    /// Cached memory of the statements visited when tracing an event.
    memory: HashSet<usize>,
    /// Maps event indices to the (indices of) statements triggered by this event.
    event_to_stmts: HashMap<usize, Vec<usize>>,
    /// Stores indices of statements triggered by events.
    eventful_calls: HashSet<usize>,
    /// Service to build isles for.
    service: &'a Service,
}
impl<'a> IsleBuilder<'a> {
    /// Factored [`Isles`] allocation.
    fn new_isles(ctx: &'a Ctx) -> Isles {
        Isles::with_capacity(ctx.count_events())
    }

    /// Constructor.
    ///
    /// The `service`'s [graph][Service::graph] must be properly setup for the builder to work.
    ///
    /// During construction, the statements of the `service` are scanned to populate a map from
    /// events to the statements that react to it.
    pub fn new(ctx: &'a Ctx, service: &'a Service, imports: &HashMap<usize, FlowImport>) -> Self {
        let real_events = Self::build_real_events(ctx, service, imports);
        let (event_to_stmts, eventful_calls) =
            Self::build_event_to_stmts(ctx, service, imports, real_events);
        Self {
            isles: Self::new_isles(ctx),
            events: HashSet::with_capacity(10),
            stack: Vec::with_capacity(service.statements.len() / 2),
            memory: HashSet::with_capacity(service.statements.len()),
            event_to_stmts,
            eventful_calls,
            service,
        }
    }

    /// Scans the statements in the `service` and produces the map from events to statements.
    ///
    /// Used by [`Self::new`].
    ///
    /// The vector of statement indices associated to any event identifier is **sorted**, *i.e.*
    /// statement indices are in the order in which they appear in the service. (It actually
    /// does not matter for the actual isle building process atm.)
    fn build_event_to_stmts(
        ctx: &Ctx,
        service: &Service,
        imports: &HashMap<usize, FlowImport>,
        real_events: HashSet<usize>,
    ) -> (HashMap<usize, Vec<usize>>, HashSet<usize>) {
        let mut map = HashMap::with_capacity(10);
        let mut set = HashSet::with_capacity(10);
        for (stmt_id, stmt) in service.statements.iter() {
            let mut triggered_by = |event: usize| {
                let vec = map.entry(event).or_insert_with(Vec::new);
                debug_assert!(!vec.contains(stmt_id));
                vec.push(*stmt_id);
                set.insert(*stmt_id);
            };

            // store events that trigger stmt
            if let Some(inputs) = stmt.try_get_call() {
                // scan incoming stmt for timers
                for import_id in service.get_dependencies(*stmt_id) {
                    if let Some(FlowImport { id: timer, .. }) = &imports.get(&import_id) {
                        if !ctx.is_service_timeout(service.id, *timer) && ctx.is_timer(*timer) {
                            // register `stmt_id` as triggered by `input`
                            triggered_by(*timer);
                        }
                    }
                }
                // scan inputs for real events
                for input in inputs {
                    if let flow::Kind::Ident { id: input } = input.1.kind {
                        if real_events.contains(&input) {
                            // register `stmt_id` as triggered by `input`
                            triggered_by(input);
                        }
                    } else {
                        todoo!("non-ident component input")
                    }
                }
            }
        }
        // all vectors in `map` should be sorted and non-empty
        debug_assert! { map.iter().all(|(_, vec)| !vec.is_empty()) }
        (map, set)
    }

    /// Scans the statements in the `service` and produces the set of real events.
    ///
    /// Used by [`Self::new`].
    ///
    /// Real events are not produced by components.
    fn build_real_events(
        ctx: &Ctx,
        service: &Service,
        imports: &HashMap<usize, FlowImport>,
    ) -> HashSet<usize> {
        let mut stack = vec![];
        let mut seen = HashSet::with_capacity(10);
        let mut real_events = HashSet::with_capacity(10);
        // add only events to 'real_events'
        let mut add_real_event = |event: usize| {
            if ctx.get_flow_kind(event).is_event() {
                real_events.insert(event);
            }
        };
        // add next stmt to 'stack' if not in 'seen'
        let mut prep_next = |stmt_id: usize, stack: &mut Vec<usize>| {
            for next in service.graph.neighbors(stmt_id) {
                if seen.insert(next) {
                    stack.push(next);
                }
            }
        };
        for (import_id, import) in imports {
            add_real_event(import.id);
            prep_next(*import_id, &mut stack);
            while let Some(stmt_id) = stack.pop() {
                if let Some(stmt) = service.statements.get(&stmt_id) {
                    if !stmt.is_comp_call() {
                        for flow_id in stmt.get_identifiers() {
                            add_real_event(flow_id);
                            prep_next(stmt_id, &mut stack);
                        }
                    }
                }
            }
        }
        real_events
    }

    /// True if `stmt` corresponds to a component call that reacts to some event.
    ///
    /// Used to stop the exploration of a dependency branch on component calls that are eventful
    /// and not triggered by the event the isle is for.
    fn is_eventful_call(&self, stmt_id: usize) -> bool {
        self.eventful_calls.contains(&stmt_id)
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
            // order does not matter that much, we can't be sure to push in the proper order and
            // the finalization in `Self::into_isles` sorts statements anyways
            self.stack
                .extend(stmts_ids.iter().map(|stmt_id| (*stmt_id, true)));
        } else {
            return;
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
