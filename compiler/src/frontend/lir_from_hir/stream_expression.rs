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
            stream::Kind::NodeApplication {
                called_node_id,
                inputs,
                ..
            } => {
                let name = symbol_table.get_name(called_node_id).clone();
                let input_fields = inputs
                    .into_iter()
                    .map(|(id, expression)| {
                        (
                            symbol_table.get_name(id).clone(),
                            expression.lir_from_hir(symbol_table),
                        )
                    })
                    .collect::<Vec<_>>();
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
