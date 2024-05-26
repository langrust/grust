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
            FlowHandler, FlowInstruction, InterfaceFlow, ServiceLoop, TimingEvent, TimingEventKind,
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
        let signals_context = self.get_signals_context(symbol_table);

        let services_loops = self.get_services_loops(symbol_table, &signals_context);

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
        signals_context: &SignalsContext,
    ) -> Vec<ServiceLoop> {
        let mut identifier_creator = IdentifierCreator::from(self.get_flows_names(symbol_table));

        let Interface {
            statements,
            mut graph,
        } = self;

        // collects components, input flows, output flows, timing events that are present in the service
        let (mut components, mut input_flows, mut output_flows, mut timing_events) =
            (vec![], vec![], vec![], vec![]);
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
                    match flow_expression.kind {
                        FlowExpressionKind::Ident { .. }
                        | FlowExpressionKind::Throtle { .. }
                        | FlowExpressionKind::OnChange { .. } => (),
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

                            // get the identifier pf the receiving flow
                            let mut flows_ids = pattern.identifiers();
                            assert!(flows_ids.len() == 1);
                            let flow_id = flows_ids.pop().unwrap();

                            // add timing_event in graph
                            graph.add_edge(fresh_id, flow_id, FlowKind::Event(Default::default()));

                            // push timing_event
                            timing_events.push(TimingEvent {
                                identifier: fresh_name,
                                kind: TimingEventKind::Period(period_ms.clone()),
                            })
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

                            // get the identifier pf the receiving flow
                            let mut flows_ids = pattern.identifiers();
                            assert!(flows_ids.len() == 1);
                            let flow_id = flows_ids.pop().unwrap();

                            // add timing_event in graph
                            graph.add_edge(fresh_id, flow_id, FlowKind::Event(Default::default()));

                            // push timing_event
                            timing_events.push(TimingEvent {
                                identifier: fresh_name,
                                kind: TimingEventKind::Timeout(deadline.clone()),
                            })
                        }
                        FlowExpressionKind::ComponentCall { component_id, .. } => {
                            // todo: add potential period constrains
                            components.push(symbol_table.get_name(component_id).clone())
                        }
                    }
                }
            };
        });

        let mut flows_handling = input_flows
            .iter()
            .map(|(id, input_flow)| {
                let InterfaceFlow { identifier, .. } = input_flow;
                let mut instructions = vec![];

                // construct subgraph starting from the input flow
                let mut subgraph = DiGraphMap::new();
                depth_first_search(&graph, Some(*id), |event| {
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

                ordered_flow_statements.into_iter().for_each(|id| {
                    // get flow statement related to id
                    let flow_statement = hash_statements.get(&id).expect("should be there");
                    // add instructions related to the nature of the statement (see the draft)
                    match flow_statement {
                        FlowStatement::Declaration(FlowDeclaration {
                            pattern,
                            flow_expression,
                            ..
                        }) => todo!(),
                        FlowStatement::Instanciation(FlowInstanciation {
                            pattern,
                            flow_expression,
                            ..
                        }) => todo!(),
                        FlowStatement::Import(_) => (), // nothing to do
                        FlowStatement::Export(_) => unreachable!(),
                    }
                    let source_name = symbol_table.get_name(id);
                    if signals_context.elements.contains_key(source_name) {
                        instructions.push(FlowInstruction::Update(source_name.clone()))
                    }
                    todo!()
                });

                FlowHandler {
                    arriving_flow: identifier.clone(),
                    instructions,
                }
            })
            .collect();

        let service_loop = ServiceLoop {
            service: String::from("toto"),
            components,
            input_flows: input_flows.into_iter().unzip::<_, _, Vec<_>, _>().1,
            timing_events,
            output_flows: output_flows.into_iter().unzip::<_, _, Vec<_>, _>().1,
            flows_handling,
        };

        vec![service_loop]
    }
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
                FlowExpressionKind::Sample { .. } | FlowExpressionKind::Throtle { .. } => {
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
                FlowExpressionKind::OnChange { flow_expression }
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
