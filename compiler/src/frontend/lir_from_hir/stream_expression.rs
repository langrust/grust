use itertools::Itertools;

prelude! {
    hir::stream,
    lir::item::Import,
}

use super::LIRFromHIR;

impl LIRFromHIR for stream::Expr {
    type LIR = lir::Expr;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        match self.kind {
            stream::Kind::Event { event_id } => {
                let name = symbol_table.get_name(event_id).clone();
                lir::Expr::InputAccess { identifier: name }
            }
            stream::Kind::NodeApplication {
                calling_node_id,
                called_node_id,
                inputs,
                ..
            } => {
                let name = symbol_table.get_name(called_node_id).clone();
                let mut input_fields = inputs
                    .into_iter()
                    .filter(|(id, _)| !symbol_table.get_type(*id).is_event())
                    .map(|(id, expression)| {
                        (
                            symbol_table.get_name(id).clone(),
                            expression.lir_from_hir(symbol_table),
                        )
                    })
                    .collect::<Vec<_>>();
                if let Some(event_id) = symbol_table.get_node_event(called_node_id) {
                    input_fields.push((
                        symbol_table.get_name(event_id).clone(),
                        lir::Expr::IntoMethod {
                            expression: Box::new(lir::Expr::InputAccess {
                                identifier: symbol_table
                                    .get_name(
                                        symbol_table
                                            .get_node_event(calling_node_id)
                                            .expect("there should be event"),
                                    )
                                    .clone(),
                            }),
                        },
                    ))
                }
                lir::Expr::NodeCall {
                    node_identifier: name.clone(),
                    input_name: to_camel_case(&format!("{name}Input")),
                    input_fields,
                }
            }
            stream::Kind::Expression { expression } => expression.lir_from_hir(symbol_table),
            stream::Kind::FollowedBy { .. } => {
                unreachable!()
            }
        }
    }

    fn get_type(&self) -> Option<&Typ> {
        self.typing.as_ref()
    }

    fn is_if_then_else(&self, symbol_table: &SymbolTable) -> bool {
        match &self.kind {
            stream::Kind::Expression { expression } => expression.is_if_then_else(symbol_table),
            _ => false,
        }
    }

    fn get_imports(&self, symbol_table: &SymbolTable) -> Vec<Import> {
        match &self.kind {
            stream::Kind::Event { .. } => vec![],
            stream::Kind::Expression { expression } => expression.get_imports(symbol_table),
            stream::Kind::NodeApplication {
                called_node_id,
                inputs,
                ..
            } => {
                let mut imports = inputs
                    .iter()
                    .flat_map(|(_, expression)| expression.get_imports(symbol_table))
                    .unique()
                    .collect::<Vec<_>>();
                imports.push(Import::StateMachine(
                    symbol_table.get_name(*called_node_id).clone(),
                ));
                imports
            }
            stream::Kind::FollowedBy { .. } => unreachable!(),
        }
    }
}
