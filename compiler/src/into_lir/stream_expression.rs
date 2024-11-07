prelude! { hir::stream::Kind }

impl IntoLir<&'_ SymbolTable> for hir::stream::Expr {
    type Lir = lir::Expr;

    fn into_lir(self, symbol_table: &SymbolTable) -> Self::Lir {
        match self.kind {
            Kind::NodeApplication {
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
                            expression.into_lir(symbol_table),
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
            Kind::Expression { expr } => expr.into_lir(symbol_table),
            Kind::SomeEvent { expr } => lir::Expr::some(expr.into_lir(symbol_table)),
            Kind::NoneEvent => lir::Expr::none(),
            Kind::FollowedBy { id, .. } => {
                let name = symbol_table.get_name(id).clone();
                lir::Expr::MemoryAccess { identifier: name }
            }
            Kind::RisingEdge { .. } => unreachable!(),
        }
    }

    fn get_typ(&self) -> Option<&Typ> {
        self.typ.as_ref()
    }

    fn is_if_then_else(&self, symbol_table: &SymbolTable) -> bool {
        match &self.kind {
            Kind::Expression { expr } => expr.is_if_then_else(symbol_table),
            _ => false,
        }
    }
}
