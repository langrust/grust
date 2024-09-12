prelude! { hir::stream }

use super::LIRFromHIR;

impl LIRFromHIR for stream::Expr {
    type LIR = lir::Expr;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        match self.kind {
            stream::Kind::NodeApplication {
                memory_id,
                called_node_id,
                inputs,
                ..
            } => {
                let memory_ident = symbol_table
                    .get_name(
                        memory_id.expect("should be defined in `hir::stream::Expr::memorize`"),
                    )
                    .clone();
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
                    memory_ident,
                    node_identifier: name.clone(),
                    input_name: to_camel_case(&format!("{name}Input")),
                    input_fields,
                }
            }
            stream::Kind::Expression { expression } => expression.lir_from_hir(symbol_table),
            stream::Kind::SomeEvent { expression } => {
                lir::Expr::some(expression.lir_from_hir(symbol_table))
            }
            stream::Kind::NoneEvent => lir::Expr::none(),
            stream::Kind::FollowedBy { .. } => unreachable!(),
            stream::Kind::RisingEdge { .. } => unreachable!(),
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
}
