use petgraph::{
    algo::toposort,
    graphmap::DiGraphMap,
    visit::{depth_first_search, DfsEvent},
    Direction,
};

prelude! {
    quote::format_ident,
    ast::interface::FlowKind,
    hir::{
        flow, IdentifierCreator, interface::{
            FlowDeclaration, FlowExport, FlowImport, FlowInstantiation, FlowStatement, Interface,
        },
    },
    lir::item::execution_machine::{
        flows_context::FlowsContext,
        service_loop::{
            ArrivingFlow, Expression, FlowHandler, FlowInstruction, InterfaceFlow, ServiceLoop,
            TimingEvent, TimingEventKind,
        },
        ExecutionMachine,
    },
}

use super::LIRFromHIR;

impl Interface {
    pub fn lir_from_hir(self, symbol_table: &mut SymbolTable) -> ExecutionMachine {
        if self.statements.is_empty() {
            return Default::default();
        }

        let mut flows_context = self.get_flows_context(symbol_table);
        let services_loops = self.get_services_loops(symbol_table, &mut flows_context);

        ExecutionMachine {
            flows_context,
            services_loops,
        }
    }
}

impl Interface {
    fn get_flows_context(&self, symbol_table: &SymbolTable) -> FlowsContext {
        let mut flows_context = FlowsContext {
            elements: Default::default(),
            event_components: Default::default(),
            components: Default::default(),
        };
        self.statements
            .iter()
            .for_each(|statement| statement.add_flows_context(&mut flows_context, symbol_table));
        flows_context
    }
    fn get_services_loops(
        self,
        symbol_table: &mut SymbolTable,
        flows_context: &mut FlowsContext,
    ) -> Vec<ServiceLoop> {
        let mut identifier_creator = IdentifierCreator::from(self.get_flows_names(symbol_table));

        let Interface {
            mut statements,
            mut graph,
        } = self;

        // collects components, input flows, output flows, timing events that are present in the service
        let mut components = vec![];
        let mut input_flows = vec![];
        let mut output_flows = vec![];
        let mut timing_events = HashMap::new();
        let mut on_change_events = HashMap::new();
        let mut new_statements = vec![];
        let mut fresh_statement_id = statements.len();
        statements
            .iter()
            .enumerate()
            .for_each(|(index, statement)| {
                match statement {
                    FlowStatement::Import(FlowImport {
                        id,
                        path,
                        flow_type,
                        ..
                    }) => {
                        input_flows.push((
                            *id,
                            InterfaceFlow {
                                path: path.clone(),
                                identifier: symbol_table.get_name(*id).clone(),
                                r#type: flow_type.clone(),
                            },
                        ));
                    }
                    FlowStatement::Export(FlowExport {
                        id,
                        path,
                        flow_type,
                        ..
                    }) => output_flows.push((
                        *id,
                        InterfaceFlow {
                            path: path.clone(),
                            identifier: symbol_table.get_name(*id).clone(),
                            r#type: flow_type.clone(),
                        },
                    )),
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
                            flow::Kind::Ident { .. } | flow::Kind::Throtle { .. } => (),
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
                                let fresh_id = symbol_table.insert_fresh_period(fresh_name.clone());

                                // add timing_event in new_statements
                                new_statements.push(FlowStatement::Import(FlowImport {
                                    import_token: Default::default(),
                                    id: fresh_id,
                                    path: format_ident!("{fresh_name}").into(),
                                    colon_token: Default::default(),
                                    flow_type: typing,
                                    semi_token: Default::default(),
                                }));
                                // add timing_event in graph
                                graph.add_edge(fresh_statement_id, index, ());
                                fresh_statement_id += 1;

                                // push timing_event
                                timing_events.insert(
                                    index,
                                    (
                                        fresh_id,
                                        TimingEvent {
                                            identifier: fresh_name,
                                            kind: TimingEventKind::Period(period_ms.clone()),
                                        },
                                    ),
                                );
                            }
                            flow::Kind::Timeout { deadline, .. } => {
                                // add new timing event into the identifier creator
                                let fresh_name = identifier_creator.fresh_identifier("timeout");
                                let typing = Typ::Event(Box::new(Typ::Time));
                                let fresh_id =
                                    symbol_table.insert_fresh_deadline(fresh_name.clone());

                                // add timing_event in new_statements
                                new_statements.push(FlowStatement::Import(FlowImport {
                                    import_token: Default::default(),
                                    id: fresh_id,
                                    path: format_ident!("{fresh_name}").into(),
                                    colon_token: Default::default(),
                                    flow_type: typing,
                                    semi_token: Default::default(),
                                }));
                                // add timing_event in graph
                                graph.add_edge(fresh_statement_id, index, ());
                                fresh_statement_id += 1;

                                // push timing_event
                                timing_events.insert(
                                    index,
                                    (
                                        fresh_id,
                                        TimingEvent {
                                            identifier: fresh_name,
                                            kind: TimingEventKind::Timeout(deadline.clone()),
                                        },
                                    ),
                                );
                            }
                            flow::Kind::ComponentCall { component_id, .. } => {
                                // add potential period constrains
                                if let Some(period) = symbol_table.get_node_period(*component_id) {
                                    // add new timing event into the identifier creator
                                    let fresh_name = identifier_creator.fresh_identifier("period");
                                    let typing = Typ::Event(Box::new(Typ::Time));
                                    let fresh_id =
                                        symbol_table.insert_fresh_period(fresh_name.clone());

                                    // add timing_event in new_statements
                                    new_statements.push(FlowStatement::Import(FlowImport {
                                        import_token: Default::default(),
                                        id: fresh_id,
                                        path: format_ident!("{fresh_name}").into(),
                                        colon_token: Default::default(),
                                        flow_type: typing,
                                        semi_token: Default::default(),
                                    }));
                                    // add timing_event in graph
                                    graph.add_edge(fresh_statement_id, index, ());
                                    fresh_statement_id += 1;

                                    // push timing_event
                                    timing_events.insert(
                                        index,
                                        (
                                            fresh_id,
                                            TimingEvent {
                                                identifier: fresh_name,
                                                kind: TimingEventKind::Period(period.clone()),
                                            },
                                        ),
                                    );
                                }
                                components.push(symbol_table.get_name(*component_id).clone())
                            }
                        }
                    }
                };
            });

        // push new_statements into statements
        statements.append(&mut new_statements);

        // for every incoming flows, compute their handlers
        let flows_handling: Vec<_> = statements
            .iter()
            .enumerate()
            .filter_map(|(index, statement)| match statement {
                FlowStatement::Import(FlowImport { id, .. }) => Some((index, *id)),
                _ => None,
            })
            .map(|(index, flow_id)| {
                // construct subgraph starting from the input flow
                let subgraph = construct_subgraph_from_source(index, &graph);
                // sort statement in dependency order
                let mut ordered_statements = toposort(&subgraph, None).expect("should succeed");
                ordered_statements.reverse();
                // if input flow is an event then store its identifier
                let (encountered_events, defined_signals) = {
                    let flow_id_set = [flow_id].into_iter().collect();
                    match symbol_table.get_flow_kind(flow_id) {
                        FlowKind::Signal(_) => (HashSet::new(), flow_id_set),
                        FlowKind::Event(_) => (flow_id_set, HashSet::new()),
                    }
                };
                // compute instructions that depend on this incoming flow
                let instructions = compute_flow_instructions(
                    vec![index],
                    &statements,
                    &subgraph,
                    &ordered_statements,
                    encountered_events,
                    defined_signals,
                    &on_change_events,
                    &timing_events,
                    flows_context,
                    symbol_table,
                );
                // determine weither this arriving flow is a timing event
                let flow_name = symbol_table.get_name(flow_id).clone();
                let arriving_flow = if symbol_table.is_period(flow_id) {
                    ArrivingFlow::Period(flow_name)
                } else if symbol_table.is_deadline(flow_id) {
                    ArrivingFlow::Deadline(flow_name)
                } else {
                    let flow_type = symbol_table.get_type(flow_id);
                    ArrivingFlow::Channel(flow_name, flow_type.clone())
                };
                // get the name of timeout events from reset instructions
                let deadline_args = instructions
                    .iter()
                    .filter_map(|instruction| match instruction {
                        FlowInstruction::ResetTimer(deadline_name, _) => {
                            Some(deadline_name.clone())
                        }
                        _ => None,
                    })
                    .collect();
                FlowHandler {
                    arriving_flow,
                    deadline_args,
                    instructions,
                }
            })
            .collect();

        let service_loop = ServiceLoop {
            service: "toto".into(),
            components,
            input_flows: input_flows.into_iter().unzip::<_, _, Vec<_>, _>().1,
            timing_events: timing_events.into_values().unzip::<_, _, Vec<_>, _>().1,
            output_flows: output_flows.into_iter().unzip::<_, _, Vec<_>, _>().1,
            flows_handling,
        };

        vec![service_loop]
    }
}

fn construct_subgraph_from_source(
    source_id: usize,
    graph: &DiGraphMap<usize, ()>,
) -> DiGraphMap<usize, ()> {
    let mut subgraph = DiGraphMap::new();
    depth_first_search(&graph, Some(source_id), |event| {
        use DfsEvent::*;
        match event {
            CrossForwardEdge(parent, child) | BackEdge(parent, child) | TreeEdge(parent, child) => {
                let weight = graph
                    .edge_weight(parent, child)
                    .expect("there must be an edge")
                    .clone();
                subgraph.add_edge(parent, child, weight);
            }
            Discover(_, _) | Finish(_, _) => {}
        }
    });
    subgraph
}

fn compute_flow_instructions(
    mut working_stack: Vec<usize>,
    statements: &Vec<FlowStatement>,
    graph: &DiGraphMap<usize, ()>,
    ordered_statements: &Vec<usize>,
    mut encountered_events: HashSet<usize>,
    mut defined_signals: HashSet<usize>,
    on_change_events: &HashMap<usize, usize>,
    timing_events: &HashMap<usize, (usize, TimingEvent)>,
    flows_context: &FlowsContext,
    symbol_table: &SymbolTable,
) -> Vec<FlowInstruction> {
    let mut instructions = vec![];
    let statements_order = ordered_statements
        .into_iter()
        .enumerate()
        .map(|(order, statement_id)| (statement_id, order))
        .collect::<HashMap<_, _>>();
    let compare_statements = |statement_id: &usize| *statements_order.get(statement_id).unwrap();

    // push instructions in right order
    while !working_stack.is_empty() {
        // get the next flow statement to transform
        let statement_id = working_stack.pop().unwrap();
        // get flow statement related to id
        let flow_statement = statements.get(statement_id).expect("should be there");

        // remember next statements should be stacked into working_stack
        let mut add_dependent_statements = false;

        // add instructions related to the nature of the statement (see the draft)
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
            }) => {
                if let flow::Kind::ComponentCall {
                    component_id,
                    inputs,
                } = &flow_expression.kind
                {
                    let component_name = symbol_table.get_name(*component_id);

                    // get outputs' ids
                    let outputs_ids = pattern.identifiers();

                    // get timing event identifier if it exists
                    if let Some((timer_id, _)) = timing_events.get(&statement_id) {
                        // if timing event is activated
                        if encountered_events.contains(timer_id) {
                            // if component computes on event
                            if symbol_table.has_events(*component_id) {
                                // call component with 'no event'
                                instructions.push(FlowInstruction::EventComponentCall(
                                    pattern.clone().lir_from_hir(symbol_table),
                                    component_name.clone(),
                                    None,
                                ));
                            } else {
                                // call component without
                                instructions.push(FlowInstruction::ComponentCall(
                                    pattern.clone().lir_from_hir(symbol_table),
                                    component_name.clone(),
                                ));
                            }

                            // propagate computations
                            add_dependent_statements = true;

                            // update output signals
                            for output_id in outputs_ids.iter() {
                                let output_name = symbol_table.get_name(*output_id);
                                instructions.push(FlowInstruction::UpdateContext(
                                    output_name.clone(),
                                    Expression::Identifier {
                                        identifier: output_name.clone(),
                                    },
                                ));
                            }
                        }
                    }

                    // get the potential event that will call the component
                    let dependencies: HashSet<usize> =
                        flow_expression.get_dependencies().into_iter().collect();
                    let mut overlapping_events = dependencies.intersection(&encountered_events);
                    debug_assert!(overlapping_events.clone().collect::<Vec<_>>().len() <= 1);

                    // if one of its dependencies is the encountered event
                    // then call component with the event and update output signals
                    if let Some(flow_event_id) = overlapping_events.next() {
                        // get event id in the component
                        let mut component_event_element_ids = inputs
                            .iter()
                            .filter_map(|(component_event_element_id, flow_expression)| {
                                match flow_expression.kind {
                                    flow::Kind::Ident { id } => {
                                        if id.eq(flow_event_id) {
                                            Some(*component_event_element_id)
                                        } else {
                                            None
                                        }
                                    }
                                    _ => unreachable!(), // normalized
                                }
                            })
                            .collect::<Vec<_>>();
                        debug_assert!(component_event_element_ids.len() == 1);
                        let component_event_element_id = component_event_element_ids.pop().unwrap();

                        // call component with the event
                        instructions.push(FlowInstruction::EventComponentCall(
                            pattern.clone().lir_from_hir(symbol_table),
                            component_name.clone(),
                            Some((
                                symbol_table.get_name(component_event_element_id).clone(),
                                symbol_table.get_name(*flow_event_id).clone(),
                            )),
                        ));

                        // propagate computations
                        add_dependent_statements = true;

                        // update output signals
                        for output_id in outputs_ids.iter() {
                            let output_name = symbol_table.get_name(*output_id);
                            instructions.push(FlowInstruction::UpdateContext(
                                output_name.clone(),
                                Expression::Identifier {
                                    identifier: output_name.clone(),
                                },
                            ));
                        }
                    }
                } else {
                    // get the id of pattern's flow, debug-check there is only one flow
                    let mut ids = pattern.identifiers();
                    debug_assert!(ids.len() == 1);
                    let id_pattern = ids.pop().unwrap();

                    // get the id of flow_expression, debug-check there is only one flow
                    let mut ids = flow_expression.get_dependencies();
                    debug_assert!(ids.len() == 1);
                    let id_source = ids.pop().unwrap();

                    let flow_name = symbol_table.get_name(id_pattern);
                    let source_name = symbol_table.get_name(id_source);

                    match &flow_expression.kind {
                        flow::Kind::Ident { .. } => {
                            // only set identifier if source is a signal or an activated event
                            match symbol_table.get_flow_kind(id_source) {
                                FlowKind::Signal(_) =>
                                // set the signal's statement
                                {
                                    // if source is a signal, look if it is defined
                                    if defined_signals.contains(&id_source) {
                                        instructions.push(FlowInstruction::Let(
                                            flow_name.clone(),
                                            Expression::Identifier {
                                                identifier: source_name.clone(),
                                            },
                                        ))
                                    } else {
                                        // if not defined, then get it from the context
                                        instructions.push(FlowInstruction::Let(
                                            flow_name.clone(),
                                            Expression::InContext {
                                                flow: source_name.clone(),
                                            },
                                        ))
                                    }
                                    // add the flow to the set of defined signals
                                    defined_signals.insert(id_pattern);

                                    // propagate computations
                                    add_dependent_statements = true;
                                }
                                FlowKind::Event(_) => {
                                    // if source is an event, look if it is activated
                                    if encountered_events.contains(&id_source) {
                                        // if activated, then rename the encountered_event
                                        // and add the 'let' instruction
                                        encountered_events.insert(id_pattern);
                                        instructions.push(FlowInstruction::Let(
                                            flow_name.clone(),
                                            Expression::Identifier {
                                                identifier: source_name.clone(),
                                            },
                                        ));

                                        // add the flow to the set of encountered events
                                        encountered_events.insert(id_pattern);

                                        // propagate computations
                                        add_dependent_statements = true;
                                    }
                                }
                            }
                        }
                        flow::Kind::Sample { .. } => {
                            let (timer_id, _) = timing_events
                                .get(&statement_id)
                                .expect("there should be a timing event");

                            // source is an event, look if it is activated
                            if encountered_events.contains(&id_source) {
                                // if activated, store event value
                                instructions.push(FlowInstruction::UpdateContext(
                                    source_name.clone(),
                                    Expression::Some {
                                        expression: Box::new(Expression::Identifier {
                                            identifier: source_name.clone(),
                                        }),
                                    },
                                ))
                            }

                            // if timing event is activated
                            if encountered_events.contains(timer_id) {
                                // if activated, update signal by taking from stored event value
                                instructions.push(FlowInstruction::UpdateContext(
                                    flow_name.clone(),
                                    Expression::TakeFromContext {
                                        flow: source_name.clone(),
                                    },
                                ));

                                // propagate computations
                                add_dependent_statements = true;
                            }
                        }
                        flow::Kind::Throtle { delta, .. } => {
                            // source is a signal, if it is not defined, then define it
                            if !defined_signals.contains(&id_source) {
                                instructions.push(FlowInstruction::Let(
                                    source_name.clone(),
                                    Expression::InContext {
                                        flow: source_name.clone(),
                                    },
                                ));

                                // add the flow to the set of defined signals
                                defined_signals.insert(id_source);
                            }

                            // update created signal
                            instructions.push(FlowInstruction::IfThrotle(
                                flow_name.clone(),
                                source_name.clone(),
                                delta.clone(),
                                Box::new(FlowInstruction::UpdateContext(
                                    flow_name.clone(),
                                    Expression::Identifier {
                                        identifier: source_name.clone(),
                                    },
                                )),
                            ));

                            // propagate computations
                            add_dependent_statements = true;
                        }
                        flow::Kind::OnChange { .. } => {
                            // source is a signal, if it is not defined, then define it
                            if !defined_signals.contains(&id_source) {
                                instructions.push(FlowInstruction::Let(
                                    source_name.clone(),
                                    Expression::InContext {
                                        flow: source_name.clone(),
                                    },
                                ));

                                // add the flow to the set of defined signals
                                defined_signals.insert(id_source);
                            }

                            // get the id of the pervious event
                            let id_old_event =
                                on_change_events.get(&id_pattern).expect("should be there");

                            let old_event_name = symbol_table.get_name(*id_old_event);

                            // if on_change event is NOT activated
                            let not_onchange_instructions = compute_flow_instructions(
                                working_stack.clone(),
                                statements,
                                graph,
                                ordered_statements,
                                encountered_events.clone(),
                                defined_signals.clone(),
                                on_change_events,
                                timing_events,
                                flows_context,
                                symbol_table,
                            );

                            // if on_change event is activated
                            encountered_events.insert(id_pattern);
                            let mut onchange_instructions = vec![
                                FlowInstruction::Let(
                                    flow_name.clone(),
                                    Expression::Identifier {
                                        identifier: source_name.clone(),
                                    },
                                ),
                                FlowInstruction::UpdateContext(
                                    old_event_name.clone(),
                                    Expression::Identifier {
                                        identifier: source_name.clone(),
                                    },
                                ),
                            ];
                            // insert statements that are dependent from on_change event
                            graph.neighbors(statement_id).for_each(|next_statement_id| {
                                // insert statements into the sorted vector 'working_stack'
                                match working_stack
                                    .binary_search_by_key(&next_statement_id, compare_statements)
                                {
                                    Err(pos) => working_stack.insert(pos, next_statement_id),
                                    Ok(_) => unreachable!(), // loop in the graph, impossible
                                }
                            });
                            let mut next_onchange_instructions = compute_flow_instructions(
                                working_stack, // takes ownership
                                statements,
                                graph,
                                ordered_statements,
                                encountered_events, // takes ownership
                                defined_signals,    // takes ownership
                                on_change_events,
                                timing_events,
                                flows_context,
                                symbol_table,
                            );
                            onchange_instructions.append(&mut next_onchange_instructions);

                            instructions.push(FlowInstruction::IfChange(
                                old_event_name.clone(),
                                source_name.clone(),
                                onchange_instructions,
                                not_onchange_instructions,
                            ));

                            // ends the loop
                            break;
                        }
                        flow::Kind::Timeout { deadline, .. } => {
                            let (timer_id, timer) = timing_events
                                .get(&statement_id)
                                .expect("there should be a timing event");
                            let timer_name = &timer.identifier;

                            // source is an event, look if it is activated
                            if encountered_events.contains(&id_source) {
                                // if activated, create event
                                encountered_events.insert(id_pattern);

                                // add event creation in instruction
                                instructions.push(FlowInstruction::Let(
                                    flow_name.clone(),
                                    Expression::Ok {
                                        expression: Box::new(Expression::Identifier {
                                            identifier: source_name.clone(),
                                        }),
                                    },
                                ));

                                // add reset timer
                                instructions.push(FlowInstruction::ResetTimer(
                                    timer_name.clone(),
                                    *deadline,
                                ));

                                // propagate computations
                                add_dependent_statements = true;
                            }

                            // timer is an event, look if it is activated
                            if encountered_events.contains(timer_id) {
                                // if activated, create event
                                encountered_events.insert(id_pattern);

                                // add event creation in instruction
                                instructions
                                    .push(FlowInstruction::Let(flow_name.clone(), Expression::Err));

                                // add reset timer
                                instructions.push(FlowInstruction::ResetTimer(
                                    timer_name.clone(),
                                    *deadline,
                                ));

                                // propagate computations
                                add_dependent_statements = true;
                            }
                        }
                        flow::Kind::Scan { .. } => {
                            let (timer_id, _) = timing_events
                                .get(&statement_id)
                                .expect("there should be a timing event");

                            // timer is an event, look if it is activated
                            if encountered_events.contains(timer_id) {
                                // if activated, create event
                                encountered_events.insert(id_pattern);

                                // add event creation in instructions
                                // source is a signal, look if it is defined
                                if defined_signals.contains(&id_source) {
                                    instructions.push(FlowInstruction::Let(
                                        flow_name.clone(),
                                        Expression::Identifier {
                                            identifier: source_name.clone(),
                                        },
                                    ))
                                } else {
                                    // if not defined, then get it from the context
                                    instructions.push(FlowInstruction::Let(
                                        flow_name.clone(),
                                        Expression::InContext {
                                            flow: source_name.clone(),
                                        },
                                    ))
                                }

                                // propagate computations
                                add_dependent_statements = true;
                            }
                        }
                        flow::Kind::ComponentCall { .. } => unreachable!(),
                    }
                }
            }
            FlowStatement::Export(FlowExport { id, .. }) => {
                let source_name = symbol_table.get_name(*id);
                // if source flow is an encountered event or a defined signal
                if encountered_events.contains(id) || defined_signals.contains(id) {
                    // add send instruction
                    instructions.push(FlowInstruction::Send(
                        source_name.clone(),
                        Expression::Identifier {
                            identifier: source_name.clone(),
                        },
                    ));
                }
            }
            FlowStatement::Import(FlowImport { id, .. }) => {
                // if flow is in context, add context_update instruction
                if flows_context.contains_element(symbol_table.get_name(*id)) {
                    let flow_name = symbol_table.get_name(*id);
                    instructions.push(FlowInstruction::UpdateContext(
                        flow_name.clone(),
                        Expression::Identifier {
                            identifier: flow_name.clone(),
                        },
                    ))
                }

                // propagate computations
                add_dependent_statements = true;
            }
        }

        // insert dependent statements if needed
        if add_dependent_statements {
            graph.neighbors(statement_id).for_each(|next_statement_id| {
                // insert statements into the sorted vector 'working_stack'
                match working_stack.binary_search_by_key(&next_statement_id, compare_statements) {
                    Err(pos) => working_stack.insert(pos, next_statement_id),
                    Ok(_) => unreachable!(), // loop in the graph, impossible
                }
            });
        }
    }

    instructions
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
                flow::Kind::Throtle { .. } => {
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

                    let mut input_fields = vec![];

                    inputs
                        .iter()
                        .filter(|(input_id, _)| !symbol_table.get_type(*input_id).is_event())
                        .filter_map(|(input_id, flow_expression)| {
                            match &flow_expression.kind {
                                // get the id of flow_expression (and check it is an idnetifier, from normalization)
                                flow::Kind::Ident { id: flow_id } => {
                                    // push input_field_name and flow_name in input_fields
                                    let input_field_name = symbol_table.get_name(*input_id).clone();
                                    let flow_name = symbol_table.get_name(*flow_id).clone();
                                    input_fields.push((input_field_name, flow_name));

                                    // only retain signals' ids
                                    match symbol_table.get_flow_kind(*flow_id) {
                                        FlowKind::Signal(_) => Some(*flow_id),
                                        FlowKind::Event(_) => None,
                                    }
                                }
                                _ => unreachable!(),
                            }
                        })
                        .for_each(|id| {
                            // push in signals context
                            let source_name = symbol_table.get_name(id).clone();
                            let ty = symbol_table.get_type(id);
                            flows_context.add_element(source_name, ty);
                        });

                    if let Some(event_id) = symbol_table.get_node_event(*component_id) {
                        flows_context.add_event_component(
                            symbol_table.get_name(*component_id).clone(),
                            input_fields,
                            symbol_table.get_name(event_id).clone(),
                        )
                    } else {
                        flows_context.add_component(
                            symbol_table.get_name(*component_id).clone(),
                            input_fields,
                        )
                    }
                }
                flow::Kind::Ident { .. }
                | flow::Kind::OnChange { .. }
                | flow::Kind::Timeout { .. } => (),
            },
            _ => (),
        }
    }
}

/// An *"isle"* for some event `e` is all (and only) the statements to run when receiving `e`.
///
/// This structure is only meant to be used immutably *after* it is created by [`IsleBuilder`], in
/// fact all `&mut self` functions on [`Isles`] are private.
///
/// Given an interface and some statements, each event triggers statements that feature call to
/// eventful component. To actually run them, we need to update their inputs, which means that we
/// need to know the event-free statements (including event-free component calls) that produce the
/// inputs for each eventful component call triggered.
///
/// The *"isle"* for event `e` is the list of statements from the interface that need to run to
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
/// The [constructor][Self::new] requires an [`Interface`] (and other things) and **will use its
/// internal [graph][Interface::graph], make sure it is properly setup.
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
    /// Interface to build isles for.
    interface: &'a Interface,
}
impl<'a> IsleBuilder<'a> {
    /// Factored [`Isles`] allocation.
    fn new_isles(_syms: &'a SymbolTable) -> Isles {
        // #TODO retrieve event count from `syms` for capacity
        Isles::with_capacity(10)
    }

    /// Constructor.
    ///
    /// The `interface`'s [graph][Interface::graph] must be properly setup for the builder to work.
    ///
    /// During construction, the statements of the `interface` are scanned to populate a map from
    /// events to the statements that react to it.
    pub fn new(syms: &'a SymbolTable, interface: &'a Interface) -> Self {
        let event_to_stmts = Self::build_event_to_stmts(syms, interface);
        Self {
            isles: Self::new_isles(syms),
            events: HashSet::with_capacity(10),
            stack: Vec::with_capacity(interface.statements.len() / 2),
            memory: HashSet::with_capacity(interface.statements.len()),
            event_to_stmts,
            syms,
            interface,
        }
    }

    /// Scans the statements in the `interface` and produces the map from events to statements.
    ///
    /// Used by [`Self::new`].
    ///
    /// The vector of statement indices associated to any event index is **sorted**, *i.e.*
    /// statement indices are in the order in which they appear in the interface. (It actually does
    /// not matter for the actual isle building process atm.)
    fn build_event_to_stmts(
        syms: &SymbolTable,
        interface: &'a Interface,
    ) -> HashMap<usize, Vec<usize>> {
        let mut map = HashMap::with_capacity(10);
        for (stmt_idx, stmt) in interface.statements.iter().enumerate() {
            let mut triggered_by = |event: usize| {
                let vec = map.entry(event).or_insert_with(Vec::new);
                debug_assert!(!vec.contains(&stmt_idx));
                vec.push(stmt_idx);
            };

            if let Some((_, inputs)) = stmt.try_get_call() {
                // scan inputs for events
                for input in inputs {
                    if let flow::Kind::Ident { id: input } = input.1.kind {
                        if syms.get_flow_kind(input).is_event() {
                            // register `stmt_idx` as triggered by `input`
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
    fn is_eventful_call(&self, stmt: usize) -> bool {
        self.interface.statements[stmt]
            .try_get_call()
            .map(|(idx, _)| self.syms.has_events(idx))
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
        if let Some(stmts) = self.event_to_stmts.get(&event) {
            debug_assert!(self.stack.is_empty());
            // order does not matter that much, we can't be sure to push in the proper order and the
            // finalization in `Self::into_isles` sorts statements anyways
            self.stack.extend(stmts.iter().map(|stmt| (*stmt, true)));
        } else {
            return ();
        }

        debug_assert!(self.isles.is_isle_empty(event));
        debug_assert!(self.memory.is_empty());
        debug_assert!(self.events.is_empty());
        let _is_new = self.events.insert(event);
        debug_assert!(_is_new);

        'stmts: while let Some((stmt, is_top)) = self.stack.pop() {
            let is_new = self.memory.insert(stmt);
            // continue if not new or we have reached an eventful call that's not at the top
            if !is_new || !is_top && self.is_eventful_call(stmt) {
                continue 'stmts;
            }

            self.isles.insert(event, stmt);

            for stmt in self
                .interface
                .graph
                .neighbors_directed(stmt, Direction::Incoming)
            {
                self.stack.push((stmt, false));
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
