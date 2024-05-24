use crate::{
    ast::interface::FlowKind,
    hir::{
        flow_expression::FlowExpressionKind,
        interface::{FlowDeclaration, FlowInstanciation, FlowStatement, Interface},
    },
    lir::item::execution_machine::{signals_context::SignalsContext, ExecutionMachine},
    symbol_table::SymbolTable,
};

use super::LIRFromHIR;

impl LIRFromHIR for Interface {
    type LIR = ExecutionMachine;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        let signals_context = self.get_signals_context(symbol_table);

        let run_loop = todo!();

        ExecutionMachine {
            signals_context,
            run_loop,
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
