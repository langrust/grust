use crate::{
    hir::flow_expression::{FlowExpression, FlowExpressionKind},
    symbol_table::SymbolTable,
};

impl FlowExpression {
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
            FlowExpressionKind::Ident { .. } => (),
            FlowExpressionKind::Sample {
                flow_expression, ..
            } => {
                flow_expression.change_node_application_into_unitary_node_application(symbol_table)
            }
            FlowExpressionKind::Merge {
                flow_expression_1,
                flow_expression_2,
            } => {
                flow_expression_1
                    .change_node_application_into_unitary_node_application(symbol_table);
                flow_expression_2
                    .change_node_application_into_unitary_node_application(symbol_table)
            }
            FlowExpressionKind::Zip {
                flow_expression_1,
                flow_expression_2,
            } => {
                flow_expression_1
                    .change_node_application_into_unitary_node_application(symbol_table);
                flow_expression_2
                    .change_node_application_into_unitary_node_application(symbol_table)
            }
            FlowExpressionKind::ComponentCall {
                component_id,
                inputs,
                signal_id,
            } => {
                let unitary_node_id = symbol_table.get_unitary_node_id(
                    symbol_table.get_name(*component_id),
                    symbol_table.get_name(*signal_id),
                );
                let used_inputs = symbol_table.get_unitary_node_used_inputs(unitary_node_id);

                inputs.retain_mut(|(input_id, expression)| {
                    expression.change_node_application_into_unitary_node_application(symbol_table);
                    *used_inputs.get(input_id).expect("should be there")
                });

                *component_id = unitary_node_id;
            }
        }
    }
}
