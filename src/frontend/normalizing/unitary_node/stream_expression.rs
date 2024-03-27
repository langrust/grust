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
                ref mut expression,
                ref constant,
            } => {
                // constant should not have node application
                debug_assert!(constant.no_any_node_application());

                expression.change_node_application_into_unitary_node_application(symbol_table)
            }
            StreamExpressionKind::NodeApplication {
                node_id,
                inputs,
                output_id,
            } => {
                let unitary_node_id = symbol_table.get_unitary_node_id(
                    symbol_table.get_name(*node_id),
                    symbol_table.get_name(*output_id),
                );
                let used_inputs = symbol_table.get_unitary_node_used_inputs(unitary_node_id);

                inputs.retain_mut(|(input_id, expression)| {
                    expression.change_node_application_into_unitary_node_application(symbol_table);
                    *used_inputs.get(input_id).expect("should be there")
                });

                self.convert_to_unitary_node_application(unitary_node_id);
            }
            StreamExpressionKind::UnitaryNodeApplication { .. } => unreachable!(),
        }
    }

    fn convert_to_unitary_node_application(&mut self, new_id: usize) {
        // create temporary replacement
        let temp = StreamExpressionKind::NodeApplication {
            node_id: 0,
            inputs: vec![],
            output_id: 0,
        };
        // compute unitary node application from the existing node application
        let new_kind = match std::mem::replace(&mut self.kind, temp) {
            StreamExpressionKind::NodeApplication {
                inputs, output_id, ..
            } => StreamExpressionKind::UnitaryNodeApplication {
                node_id: new_id,
                inputs,
                output_id,
            },
            _ => unreachable!(),
        };
        // reset the unitary node application
        let _ = std::mem::replace(&mut self.kind, new_kind);
    }
}
