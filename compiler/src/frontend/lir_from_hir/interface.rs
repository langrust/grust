use std::collections::HashMap;

use petgraph::{
    algo::toposort,
    graphmap::DiGraphMap,
    visit::{depth_first_search, DfsEvent},
};

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
        service_loop::{
            Expression, FlowHandler, FlowInstruction, InterfaceFlow, Pattern, ServiceLoop,
            TimingEvent, TimingEventKind,
        },
        signals_context::SignalsContext,
        ExecutionMachine,
    },
    symbol_table::SymbolTable,
};

use super::LIRFromHIR;

impl LIRFromHIR for Interface {
    type LIR = ExecutionMachine;

    fn lir_from_hir(self, symbol_table: &mut SymbolTable) -> Self::LIR {
        let mut signals_context = self.get_signals_context(symbol_table);

        let services_loops = self.get_services_loops(symbol_table, &mut signals_context);

        ExecutionMachine {
            signals_context,
            services_loops,
        }
    }
}

impl Interface {
    fn get_signals_context(&self, symbol_table: &SymbolTable) -> SignalsContext {
        let mut signals_context = SignalsContext {
            elements: Default::default(),
        };
        self.statements.iter().for_each(|statement| {
            statement.add_signals_context(&mut signals_context, symbol_table)
        });
        signals_context
    }
    fn get_services_loops(
        self,
        symbol_table: &mut SymbolTable,
        signals_context: &mut SignalsContext,
    ) -> Vec<ServiceLoop> {
        let mut identifier_creator = IdentifierCreator::from(self.get_flows_names(symbol_table));

        let Interface {
            statements,
            mut graph,
        } = self;

        // collects components, input flows, output flows, timing events that are present in the service
        let (mut components, mut input_flows, mut output_flows) = (vec![], vec![], vec![]);
        let mut timing_events = HashMap::new();
        let mut on_change_events = HashMap::new();
        let mut hash_statements = HashMap::new();
        statements.into_iter().for_each(|statement| {
            match &statement {
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
                    hash_statements.insert(*id, statement);
                }
                FlowStatement::Export(FlowExport {
                    id,
                    path,
                    flow_type,
                    ..
                }) => {
                    output_flows.push((
                        *id,
                        InterfaceFlow {
                            path: path.clone(),
                            identifier: symbol_table.get_name(*id).clone(),
                            r#type: flow_type.clone(),
                        },
                    ))
                    // do not put export statements in hash because already in instanciation
                }
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
                    pattern.identifiers().into_iter().for_each(|id| {
                        hash_statements.insert(id, statement.clone());
                    });
                    match &flow_expression.kind {
                        FlowExpressionKind::Ident { .. } | FlowExpressionKind::Throtle { .. } => (),
                        FlowExpressionKind::OnChange { flow_expression } => {
                            // get the identifier of the created event
                            let mut ids = pattern.identifiers();
                            assert!(ids.len() == 1);
                            let event_id = ids.pop().unwrap();
                            let event_name = symbol_table.get_name(event_id).clone();

                            // add new event into the identifier creator
                            let fresh_name = identifier_creator.new_identifier(
                                event_name,
                                String::from("old"),
                                String::from(""),
                            );
                            let typing = symbol_table.get_type(event_id).clone();
                            let fresh_id =
                                symbol_table.insert_fresh_flow(fresh_name.clone(), typing.clone());

                            // add event_old in signals_context
                            signals_context.elements.insert(fresh_name, typing);

                            // push in on_change_events
                            on_change_events.insert(event_id, fresh_id);
                        }
                        FlowExpressionKind::Sample { period_ms, .. }
                        | FlowExpressionKind::Scan { period_ms, .. } => {
                            // add new timing event into the identifier creator
                            let fresh_name = identifier_creator.new_identifier(
                                String::from(""),
                                String::from("period"),
                                String::from(""),
                            );
                            let typing = Type::Event(Box::new(Type::Unit));
                            let fresh_id =
                                symbol_table.insert_fresh_flow(fresh_name.clone(), typing);

                            // get the identifier of the receiving flow
                            let mut flows_ids = pattern.identifiers();
                            assert!(flows_ids.len() == 1);
                            let flow_id = flows_ids.pop().unwrap();

                            // add timing_event in graph
                            graph.add_edge(fresh_id, flow_id, FlowKind::Event(Default::default()));

                            // push timing_event
                            timing_events.insert(
                                flow_id,
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
                            let typing = Type::Event(Box::new(Type::Unit));
                            let fresh_id =
                                symbol_table.insert_fresh_flow(fresh_name.clone(), typing);

                            // get the identifier of the receiving flow
                            let mut flows_ids = pattern.identifiers();
                            assert!(flows_ids.len() == 1);
                            let flow_id = flows_ids.pop().unwrap();

                            // add timing_event in graph
                            graph.add_edge(fresh_id, flow_id, FlowKind::Event(Default::default()));

                            // push timing_event
                            timing_events.insert(
                                flow_id,
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
                            // todo: add potential period constrains
                            components.push(symbol_table.get_name(*component_id).clone())
                        }
                    }
                }
            };
        });

        let mut flows_handling: Vec<_> = input_flows
            .iter()
            .map(|(id, input_flow)| {
                let InterfaceFlow { identifier, .. } = input_flow;

                // construct subgraph starting from the input flow
                let subgraph = construct_subgraph_from_source(*id, &graph);
                // sort statement in dependency order
                let ordered_flow_statements = toposort(&subgraph, None).expect("should succeed");

                let instructions = compute_flow_instructions_from_ordered_flow_statements(
                    &hash_statements,
                    &on_change_events,
                    &timing_events,
                    ordered_flow_statements,
                    signals_context,
                    symbol_table,
                );
                FlowHandler {
                    arriving_flow: identifier.clone(),
                    instructions,
                }
            })
            .collect();

        let mut other_flows_handling: Vec<_> = timing_events
            .iter()
            .map(|(_, (timing_id, timing_event))| {
                let TimingEvent { identifier, .. } = timing_event;
                let mut instructions = vec![];

                // construct subgraph starting from the timing flow
                let mut subgraph = DiGraphMap::new();
                depth_first_search(&graph, Some(*timing_id), |event| {
                    use DfsEvent::*;
                    match event {
                        CrossForwardEdge(parent, child)
                        | BackEdge(parent, child)
                        | TreeEdge(parent, child) => {
                            let weight = graph
                                .edge_weight(parent, child)
                                .expect("there must be an edge")
                                .clone();
                            subgraph.add_edge(parent, child, weight);
                        }
                        Discover(_, _) | Finish(_, _) => {}
                    }
                });
                // sort statement in dependency order
                let ordered_flow_statements = toposort(&subgraph, None).expect("should succeed");

                // push instructions in order
                ordered_flow_statements.into_iter().for_each(|id| {
                    // get flow statement related to id (if it exists)
                    if let Some(flow_statement) = hash_statements.get(&id) {
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
                            }) => match &flow_expression.kind {
                                FlowExpressionKind::Ident { .. }
                                | FlowExpressionKind::Throtle { .. }
                                | FlowExpressionKind::OnChange { .. } => (),
                                FlowExpressionKind::Sample {
                                    flow_expression, ..
                                } => {
                                    // get the id of pattern's flow (and check their is only one flow)
                                    let mut ids = pattern.identifiers();
                                    assert!(ids.len() == 1);
                                    let id_pattern = ids.pop().unwrap();

                                    // get the id of flow_expression (and check their is only one flow)
                                    let mut ids = flow_expression.get_dependencies();
                                    assert!(ids.len() == 1);
                                    let id_source = ids.pop().unwrap();

                                    // update created signal
                                    instructions.push(FlowInstruction::Let(
                                        Pattern::Identifier(
                                            symbol_table.get_name(id_pattern).clone(),
                                        ),
                                        Expression::TakeFromContext {
                                            flow: symbol_table.get_name(id_source).clone(),
                                        },
                                    ))
                                }
                                FlowExpressionKind::Scan {
                                    flow_expression, ..
                                } => {
                                    // get the id of pattern's flow (and check their is only one flow)
                                    let mut ids = pattern.identifiers();
                                    assert!(ids.len() == 1);
                                    let id_pattern = ids.pop().unwrap();

                                    // get the id of flow_expression (and check their is only one flow)
                                    let mut ids = flow_expression.get_dependencies();
                                    assert!(ids.len() == 1);
                                    let id_source = ids.pop().unwrap();

                                    // update created signal
                                    instructions.push(FlowInstruction::Let(
                                        Pattern::Identifier(
                                            symbol_table.get_name(id_pattern).clone(),
                                        ),
                                        Expression::InContext {
                                            flow: symbol_table.get_name(id_source).clone(),
                                        },
                                    ))
                                }
                                FlowExpressionKind::Timeout { deadline, .. } => {
                                    // get the id of pattern's flow (and check their is only one flow)
                                    let mut ids = pattern.identifiers();
                                    assert!(ids.len() == 1);
                                    let id_pattern = ids.pop().unwrap();

                                    // create event
                                    instructions.push(FlowInstruction::Let(
                                        Pattern::Identifier(
                                            symbol_table.get_name(id_pattern).clone(),
                                        ),
                                        Expression::Err,
                                    ));
                                    // add reset timer
                                    let timer_name = timing_events
                                        .get(&id_pattern)
                                        .expect("there should be a timing event")
                                        .1
                                        .identifier
                                        .clone();
                                    instructions
                                        .push(FlowInstruction::ResetTimer(timer_name, *deadline));
                                }
                                FlowExpressionKind::ComponentCall { .. } => {
                                    // todo add period call
                                }
                            },
                            FlowStatement::Import(FlowImport { id, .. }) => {
                                // add a flow update if it is in the context
                                let flow_name = symbol_table.get_name(*id);
                                if signals_context.elements.contains_key(flow_name) {
                                    instructions.push(FlowInstruction::Let(
                                        Pattern::InContext(flow_name.clone()),
                                        Expression::Identifier {
                                            identifier: flow_name.clone(),
                                        },
                                    ))
                                }
                            }
                            FlowStatement::Export(_) => unreachable!(),
                        }
                    }
                });

                FlowHandler {
                    arriving_flow: identifier.clone(),
                    instructions,
                }
            })
            .collect();

        flows_handling.append(&mut other_flows_handling);

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
    source_flow_id: usize,
    graph: &DiGraphMap<usize, FlowKind>,
) -> DiGraphMap<usize, FlowKind> {
    let mut subgraph = DiGraphMap::new();
    depth_first_search(&graph, Some(source_flow_id), |event| {
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

fn compute_flow_instructions_from_ordered_flow_statements(
    statements: &HashMap<usize, FlowStatement>,
    on_change_events: &HashMap<usize, usize>,
    timing_events: &HashMap<usize, (usize, TimingEvent)>,
    mut ordered_flow_statements: Vec<usize>,
    signals_context: &SignalsContext,
    symbol_table: &SymbolTable,
) -> Vec<FlowInstruction> {
    let mut instructions = vec![];

    // push instructions in right order
    while !ordered_flow_statements.is_empty() {
        // get the next flow statement to transform
        let id = ordered_flow_statements.pop().unwrap();
        // get flow statement related to id
        let flow_statement = statements.get(&id).expect("should be there");

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
            }) => match &flow_expression.kind {
                FlowExpressionKind::Ident { id } => {
                    // get the id of pattern's flow (and check their is only one flow)
                    let mut ids = pattern.identifiers();
                    assert!(ids.len() == 1);
                    let id_pattern = ids.pop().unwrap();

                    instructions.push(FlowInstruction::Let(
                        Pattern::Identifier(symbol_table.get_name(id_pattern).clone()),
                        Expression::Identifier {
                            identifier: symbol_table.get_name(*id).clone(),
                        },
                    ));
                }
                FlowExpressionKind::Sample {
                    flow_expression, ..
                } => {
                    // get the id of pattern's flow (and check their is only one flow)
                    let mut ids = pattern.identifiers();
                    assert!(ids.len() == 1);
                    let id_pattern = ids.pop().unwrap();

                    // get the id of flow_expression (and check their is only one flow)
                    let mut ids = flow_expression.get_dependencies();
                    assert!(ids.len() == 1);
                    let id_source = ids.pop().unwrap();

                    let flow_name = symbol_table.get_name(id_pattern);
                    let source_name = symbol_table.get_name(id_source);

                    // set the signal's statement
                    instructions.push(FlowInstruction::Let(
                        Pattern::Identifier(flow_name.clone()),
                        Expression::Some {
                            expression: Box::new(Expression::Identifier {
                                identifier: source_name.clone(),
                            }),
                        },
                    ))
                }
                FlowExpressionKind::Throtle {
                    flow_expression,
                    delta,
                } => {
                    // get the id of pattern's flow (and check their is only one flow)
                    let mut ids = pattern.identifiers();
                    assert!(ids.len() == 1);
                    let id_pattern = ids.pop().unwrap();

                    // get the id of flow_expression (and check their is only one flow)
                    let mut ids = flow_expression.get_dependencies();
                    assert!(ids.len() == 1);
                    let id_source = ids.pop().unwrap();

                    let flow_name = symbol_table.get_name(id_pattern);
                    let source_name = symbol_table.get_name(id_source);

                    // update created signal
                    instructions.push(FlowInstruction::IfThortle(
                        flow_name.clone(),
                        source_name.clone(),
                        delta.clone(),
                        vec![FlowInstruction::Let(
                            Pattern::InContext(flow_name.clone()),
                            Expression::Identifier {
                                identifier: source_name.clone(),
                            },
                        )],
                    ));

                    // set the signal's statement
                    instructions.push(FlowInstruction::Let(
                        Pattern::Identifier(flow_name.clone()),
                        Expression::InContext {
                            flow: flow_name.clone(),
                        },
                    ));
                }
                FlowExpressionKind::OnChange { flow_expression } => {
                    // get the id of pattern's flow (and check their is only one flow)
                    let mut ids = pattern.identifiers();
                    assert!(ids.len() == 1);
                    let id_pattern = ids.pop().unwrap();

                    // get the id of the pervious event
                    let id_old_event = on_change_events.get(&id_pattern).expect("should be there");

                    // get the id of flow_expression (and check their is only one flow)
                    let mut ids = flow_expression.get_dependencies();
                    assert!(ids.len() == 1);
                    let id_source = ids.pop().unwrap();

                    let old_event_name = symbol_table.get_name(*id_old_event);
                    let source_name = symbol_table.get_name(id_source);

                    // create event
                    instructions.push(FlowInstruction::IfChange(
                        old_event_name.clone(),
                        source_name.clone(),
                        vec![
                            FlowInstruction::Let(
                                Pattern::Identifier(symbol_table.get_name(id_pattern).clone()),
                                Expression::Identifier {
                                    identifier: source_name.clone(),
                                },
                            ),
                            FlowInstruction::Let(
                                Pattern::InContext(old_event_name.clone()),
                                Expression::Identifier {
                                    identifier: source_name.clone(),
                                },
                            ),
                        ],
                    ))
                }
                FlowExpressionKind::Timeout {
                    flow_expression,
                    deadline,
                } => {
                    // get the id of pattern's flow (and check their is only one flow)
                    let mut ids = pattern.identifiers();
                    assert!(ids.len() == 1);
                    let id_pattern = ids.pop().unwrap();

                    // get the id of flow_expression (and check their is only one flow)
                    let mut ids = flow_expression.get_dependencies();
                    assert!(ids.len() == 1);
                    let id_source = ids.pop().unwrap();

                    // create event
                    instructions.push(FlowInstruction::Let(
                        Pattern::Identifier(symbol_table.get_name(id_pattern).clone()),
                        Expression::Ok {
                            expression: Box::new(Expression::Identifier {
                                identifier: symbol_table.get_name(id_source).clone(),
                            }),
                        },
                    ));
                    // add reset timer
                    let timer_name = timing_events
                        .get(&id_pattern)
                        .expect("there should be a timing event")
                        .1
                        .identifier
                        .clone();
                    instructions.push(FlowInstruction::ResetTimer(timer_name, *deadline));
                }
                FlowExpressionKind::ComponentCall { component_id, .. } => instructions.push(
                    FlowInstruction::ComponentCall(symbol_table.get_name(*component_id).clone()),
                ),
                FlowExpressionKind::Scan { .. } => (), // nothing to do
            },
            FlowStatement::Import(FlowImport { .. }) => (), // nothing to do
            FlowStatement::Export(_) => unreachable!(),
        }

        // add a context update if necessary
        let flow_name = symbol_table.get_name(id);
        if signals_context.elements.contains_key(flow_name) {
            instructions.push(FlowInstruction::Let(
                Pattern::InContext(flow_name.clone()),
                Expression::Identifier {
                    identifier: flow_name.clone(),
                },
            ))
        }
    }
    instructions
}

impl FlowStatement {
    fn add_signals_context(
        &self,
        signals_context: &mut SignalsContext,
        symbol_table: &SymbolTable,
    ) {
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
                    assert!(ids.len() == 1);
                    let id = ids.pop().unwrap();

                    // push in signals context
                    let name = symbol_table.get_name(id).clone();
                    let ty = symbol_table.get_type(id);
                    match signals_context.elements.insert(name, ty.clone()) {
                        Some(other_ty) => assert!(other_ty.eq(ty)),
                        None => (),
                    }
                }
                FlowExpressionKind::Sample {
                    flow_expression, ..
                }
                | FlowExpressionKind::OnChange { flow_expression }
                | FlowExpressionKind::Scan {
                    flow_expression, ..
                } => {
                    // get the id of flow_expression (and check it is an idnetifier, from normalization)
                    let id = match &flow_expression.kind {
                        FlowExpressionKind::Ident { id } => *id,
                        _ => unreachable!(),
                    };

                    // push in signals context
                    let name = symbol_table.get_name(id).clone();
                    let ty = symbol_table.get_type(id);
                    match signals_context.elements.insert(name, ty.clone()) {
                        Some(other_ty) => assert!(other_ty.eq(ty)),
                        None => (),
                    }
                }
                FlowExpressionKind::ComponentCall { inputs, .. } => inputs
                    .iter()
                    .filter_map(|(_, flow_expression)| {
                        // get the id of flow_expression (and check it is an idnetifier, from normalization)
                        // but only if they are signals
                        match &flow_expression.kind {
                            FlowExpressionKind::Ident { id } => {
                                match symbol_table.get_flow_kind(*id) {
                                    FlowKind::Signal(_) => Some(*id),
                                    FlowKind::Event(_) => None,
                                }
                            }
                            _ => unreachable!(),
                        }
                    })
                    .for_each(|id| {
                        // push in signals context
                        let name = symbol_table.get_name(id).clone();
                        let ty = symbol_table.get_type(id);
                        match signals_context.elements.insert(name, ty.clone()) {
                            Some(other_ty) => assert!(other_ty.eq(ty)),
                            None => (),
                        }
                    }),
                FlowExpressionKind::Ident { .. } | FlowExpressionKind::Timeout { .. } => (),
            },
            _ => (),
        }
    }
}
