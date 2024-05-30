use std::collections::{HashMap, HashSet};

use petgraph::{
    algo::toposort,
    graphmap::DiGraphMap,
    visit::{depth_first_search, DfsEvent},
};
use quote::format_ident;

use crate::{
    ast::interface::FlowKind,
    common::r#type::Type,
    hir::{
        flow_expression::FlowExpressionKind,
        identifier_creator::IdentifierCreator,
        interface::{
            FlowDeclaration, FlowExport, FlowImport, FlowInstanciation, FlowStatement, Interface,
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
    symbol_table::SymbolTable,
};

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
                    | FlowStatement::Instanciation(FlowInstanciation {
                        pattern,
                        flow_expression,
                        ..
                    }) => {
                        match &flow_expression.kind {
                            FlowExpressionKind::Ident { .. }
                            | FlowExpressionKind::Throtle { .. } => (),
                            FlowExpressionKind::OnChange { .. } => {
                                // get the identifier of the created event
                                let mut ids = pattern.identifiers();
                                debug_assert!(ids.len() == 1);
                                let flow_event_id = ids.pop().unwrap();
                                let event_name = symbol_table.get_name(flow_event_id).clone();

                                // add new event into the identifier creator
                                let fresh_name = identifier_creator.new_identifier(
                                    String::from(""),
                                    event_name,
                                    String::from("old"),
                                );
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
                            FlowExpressionKind::Sample { period_ms, .. }
                            | FlowExpressionKind::Scan { period_ms, .. } => {
                                // add new timing event into the identifier creator
                                let fresh_name = identifier_creator.new_identifier(
                                    String::from(""),
                                    String::from("period"),
                                    String::from(""),
                                );
                                let typing = Type::Event(Box::new(Type::Time));
                                let kind = FlowKind::Event(Default::default());
                                let fresh_id = symbol_table.insert_fresh_flow(
                                    fresh_name.clone(),
                                    kind,
                                    typing,
                                );

                                // add timing_event in new_statements
                                new_statements.push(FlowStatement::Import(FlowImport {
                                    import_token: Default::default(),
                                    id: fresh_id,
                                    path: format_ident!("{fresh_name}").into(),
                                    colon_token: Default::default(),
                                    flow_type: Type::Event(Box::new(Type::Time)),
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
                            FlowExpressionKind::Timeout { deadline, .. } => {
                                // add new timing event into the identifier creator
                                let fresh_name = identifier_creator.new_identifier(
                                    String::from(""),
                                    String::from("timeout"),
                                    String::from(""),
                                );
                                let typing = Type::Event(Box::new(Type::Time));
                                let kind = FlowKind::Event(Default::default());
                                let fresh_id = symbol_table.insert_fresh_flow(
                                    fresh_name.clone(),
                                    kind,
                                    typing,
                                );

                                // add timing_event in new_statements
                                new_statements.push(FlowStatement::Import(FlowImport {
                                    import_token: Default::default(),
                                    id: fresh_id,
                                    path: format_ident!("{fresh_name}").into(),
                                    colon_token: Default::default(),
                                    flow_type: Type::Event(Box::new(Type::Time)),
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
                            FlowExpressionKind::ComponentCall { component_id, .. } => {
                                // add potential period constrains
                                if let Some(period) = symbol_table.get_node_period(*component_id) {
                                    // add new timing event into the identifier creator
                                    let fresh_name = identifier_creator.new_identifier(
                                        String::from(""),
                                        String::from("period"),
                                        String::from(""),
                                    );
                                    let typing = Type::Event(Box::new(Type::Time));
                                    let kind = FlowKind::Event(Default::default());
                                    let fresh_id = symbol_table.insert_fresh_flow(
                                        fresh_name.clone(),
                                        kind,
                                        typing,
                                    );

                                    // add timing_event in new_statements
                                    new_statements.push(FlowStatement::Import(FlowImport {
                                        import_token: Default::default(),
                                        id: fresh_id,
                                        path: format_ident!("{fresh_name}").into(),
                                        colon_token: Default::default(),
                                        flow_type: Type::Event(Box::new(Type::Time)),
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
                let ordered_statements = toposort(&subgraph, None).expect("should succeed");
                // if input flow is an event then store its identifier
                let encountered_events = match symbol_table.get_flow_kind(flow_id) {
                    FlowKind::Signal(_) => HashSet::new(),
                    FlowKind::Event(_) => HashSet::from([flow_id]),
                };
                // compute instructions that depend on this incoming flow
                let instructions = compute_flow_instructions(
                    &statements,
                    &on_change_events,
                    &timing_events,
                    encountered_events,
                    ordered_statements,
                    flows_context,
                    symbol_table,
                );
                // determine weither this arriving flow is a timing event
                let flow_name = symbol_table.get_name(flow_id).clone();
                let arriving_flow = if symbol_table.is_time_flow(flow_id) {
                    ArrivingFlow::TimingEvent(flow_name)
                } else {
                    ArrivingFlow::Channel(flow_name)
                };
                FlowHandler {
                    arriving_flow,
                    instructions,
                }
            })
            .collect();

        let service_loop = ServiceLoop {
            service: String::from("toto"),
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
    statements: &Vec<FlowStatement>,
    on_change_events: &HashMap<usize, usize>,
    timing_events: &HashMap<usize, (usize, TimingEvent)>,
    mut encountered_events: HashSet<usize>,
    mut ordered_statements: Vec<usize>,
    flows_context: &FlowsContext,
    symbol_table: &SymbolTable,
) -> Vec<FlowInstruction> {
    let mut instructions = vec![];

    // push instructions in right order
    while !ordered_statements.is_empty() {
        // get the next flow statement to transform
        let ordered_statement_id = ordered_statements.remove(0);
        // get flow statement related to id
        let flow_statement = statements
            .get(ordered_statement_id)
            .expect("should be there");

        // add instructions related to the nature of the statement (see the draft)
        match flow_statement {
            FlowStatement::Declaration(FlowDeclaration {
                pattern,
                flow_expression,
                ..
            })
            | FlowStatement::Instanciation(FlowInstanciation {
                pattern,
                flow_expression,
                ..
            }) => {
                if let FlowExpressionKind::ComponentCall {
                    component_id,
                    inputs,
                } = &flow_expression.kind
                {
                    let component_name = symbol_table.get_name(*component_id);

                    // get outputs' ids
                    let outputs_ids = pattern.identifiers();

                    // get timing event identifier if it exists
                    if let Some((timer_id, _)) = timing_events.get(&ordered_statement_id) {
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
                                    FlowExpressionKind::Ident { id } => {
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

                    // define output signals in any case
                    for output_id in outputs_ids.iter() {
                        let output_name = symbol_table.get_name(*output_id);
                        instructions.push(FlowInstruction::Let(
                            output_name.clone(),
                            Expression::InContext {
                                flow: output_name.clone(),
                            },
                        ));
                    }
                } else {
                    // get the id of pattern's flow (and check their is only one flow)
                    let mut ids = pattern.identifiers();
                    debug_assert!(ids.len() == 1);
                    let id_pattern = ids.pop().unwrap();

                    // get the id of flow_expression (and check their is only one flow)
                    let mut ids = flow_expression.get_dependencies();
                    debug_assert!(ids.len() == 1);
                    let id_source = ids.pop().unwrap();

                    let flow_name = symbol_table.get_name(id_pattern);
                    let source_name = symbol_table.get_name(id_source);

                    match &flow_expression.kind {
                        FlowExpressionKind::Ident { .. } => {
                            // only set identifier if source is a signal or an activated event
                            match symbol_table.get_flow_kind(id_source) {
                                FlowKind::Signal(_) =>
                                // set the signal's statement
                                {
                                    instructions.push(FlowInstruction::Let(
                                        flow_name.clone(),
                                        Expression::Identifier {
                                            identifier: source_name.clone(),
                                        },
                                    ))
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
                                        ))
                                    }
                                    // if not activated, do nothing
                                }
                            }
                        }
                        FlowExpressionKind::Sample { .. } => {
                            let (timer_id, _) = timing_events
                                .get(&id_pattern)
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
                                ))
                            }

                            // define signal in any case
                            instructions.push(FlowInstruction::Let(
                                flow_name.clone(),
                                Expression::InContext {
                                    flow: flow_name.clone(),
                                },
                            ))
                        }
                        FlowExpressionKind::Throtle { delta, .. } => {
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

                            // set the signal's statement
                            instructions.push(FlowInstruction::Let(
                                flow_name.clone(),
                                Expression::InContext {
                                    flow: flow_name.clone(),
                                },
                            ));
                        }
                        FlowExpressionKind::OnChange { .. } => {
                            // get the id of the pervious event
                            let id_old_event =
                                on_change_events.get(&id_pattern).expect("should be there");

                            let old_event_name = symbol_table.get_name(*id_old_event);

                            // if on_change event is NOT activated
                            let not_onchange_instructions = compute_flow_instructions(
                                statements,
                                on_change_events,
                                timing_events,
                                encountered_events.clone(),
                                ordered_statements.clone(),
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
                            let mut next_onchange_instructions = compute_flow_instructions(
                                statements,
                                on_change_events,
                                timing_events,
                                encountered_events, // takes ownership
                                ordered_statements, // takes ownership
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
                        FlowExpressionKind::Timeout { deadline, .. } => {
                            let (timer_id, timer) = timing_events
                                .get(&id_pattern)
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
                            }
                        }
                        FlowExpressionKind::Scan { .. } => {
                            let (timer_id, _) = timing_events
                                .get(&id_pattern)
                                .expect("there should be a timing event");

                            // timer is an event, look if it is activated
                            if encountered_events.contains(timer_id) {
                                // if activated, create event
                                encountered_events.insert(id_pattern);
                                // add event creation in instructions
                                instructions.push(FlowInstruction::Let(
                                    flow_name.clone(),
                                    Expression::InContext {
                                        flow: source_name.clone(),
                                    },
                                ))
                            }
                        }
                        FlowExpressionKind::ComponentCall { .. } => unreachable!(),
                    }
                }
            }
            FlowStatement::Export(FlowExport { id, .. }) => {
                let flow_name = symbol_table.get_name(*id);
                // add send instructions if necessary
                instructions.push(FlowInstruction::Send(
                    flow_name.clone(),
                    Expression::Identifier {
                        identifier: flow_name.clone(),
                    },
                ))
            }
            FlowStatement::Import(_) => (), // nothing to do
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
            | FlowStatement::Instanciation(FlowInstanciation {
                pattern,
                flow_expression,
                ..
            }) => match &flow_expression.kind {
                FlowExpressionKind::Throtle { .. } => {
                    // get the id of pattern's flow (and check their is only one flow)
                    let mut ids = pattern.identifiers();
                    debug_assert!(ids.len() == 1);
                    let id = ids.pop().unwrap();

                    // push in signals context
                    let name = symbol_table.get_name(id).clone();
                    let ty = symbol_table.get_type(id);
                    flows_context.add_element(name, ty);
                }
                FlowExpressionKind::Sample {
                    flow_expression, ..
                } => {
                    // get the id of flow_expression (and check it is an idnetifier, from normalization)
                    let id = match &flow_expression.kind {
                        FlowExpressionKind::Ident { id } => *id,
                        _ => unreachable!(),
                    };
                    // get pattern's id
                    let mut ids = pattern.identifiers();
                    assert!(ids.len() == 1);
                    let pattern_id = ids.pop().unwrap();

                    // push in signals context
                    let source_name = symbol_table.get_name(id).clone();
                    let flow_name = symbol_table.get_name(pattern_id).clone();
                    let ty = Type::SMEvent(Box::new(symbol_table.get_type(id).clone()));
                    flows_context.add_element(source_name, &ty);
                    flows_context.add_element(flow_name, &ty);
                }
                FlowExpressionKind::Scan {
                    flow_expression, ..
                } => {
                    // get the id of flow_expression (and check it is an idnetifier, from normalization)
                    let id = match &flow_expression.kind {
                        FlowExpressionKind::Ident { id } => *id,
                        _ => unreachable!(),
                    };

                    // push in signals context
                    let source_name = symbol_table.get_name(id).clone();
                    let ty = symbol_table.get_type(id);
                    flows_context.add_element(source_name, ty);
                }
                FlowExpressionKind::ComponentCall {
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
                                FlowExpressionKind::Ident { id: flow_id } => {
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
                FlowExpressionKind::Ident { .. }
                | FlowExpressionKind::OnChange { .. }
                | FlowExpressionKind::Timeout { .. } => (),
            },
            _ => (),
        }
    }
}
