use crate::{
    hir::stream_expression::{StreamExpression, StreamExpressionKind},
    symbol_table::SymbolTable,
};

impl StreamExpression {
    /// Change node application expressions into unitary node application.
    ///
    /// It removes unused inputs from unitary node application.
    ///
    /// # Example
    ///
    /// Let be a node `my_node` as follows:
    ///
    /// ```GR
    /// node my_node(x: int, y: int) {
    ///     out o1: int = x+y;
    ///     out o2: int = 2*y;
    /// }
    /// ```
    ///
    /// The application of the node `my_node(g-1, v).o2` is changed
    /// to the application of the unitary node `my_node(v).o2`
    pub fn change_node_application_into_unitary_node_application(
        &mut self,
        symbol_table: &SymbolTable,
    ) {
        match &mut self.kind {
            StreamExpressionKind::Expression { expression } => {
                expression.change_node_application_into_unitary_node_application(symbol_table)
            }
            StreamExpressionKind::FollowedBy {
                ref mut expression, ..
            } => expression.change_node_application_into_unitary_node_application(symbol_table),
            StreamExpressionKind::NodeApplication {
                node_id,
                inputs,
                output_id,
            } => {
                let unitary_node_id = symbol_table.get_unitary_node_id(
                    symbol_table.get_name(node_id),
                    symbol_table.get_name(output_id),
                );
                let used_inputs = symbol_table.get_unitary_node_used_inputs(&unitary_node_id);

                let inputs = inputs
                    .iter_mut()
                    .zip(used_inputs)
                    .filter_map(|((input_id, expression), used)| {
                        expression
                            .change_node_application_into_unitary_node_application(symbol_table);
                        if used {
                            Some((*input_id, expression.clone()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                self.kind = StreamExpressionKind::UnitaryNodeApplication {
                    node_id: unitary_node_id,
                    inputs,
                    output_id: *output_id,
                };
            }
            StreamExpressionKind::UnitaryNodeApplication { .. } => unreachable!(),
        }
    }
}
