use crate::hir::stream_expression::{StreamExpression, StreamExpressionKind};

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
    pub fn change_node_application_into_unitary_node_application(&mut self) {
        match self.kind {
            StreamExpressionKind::FollowedBy { mut expression, .. } => {
                expression.change_node_application_into_unitary_node_application()
            }
            StreamExpressionKind::NodeApplication {
                node_id,
                inputs,
                output_id,
            } => {
                let unitary_node_id: usize = todo!("get id from symbol table");
                let used_inputs: Vec<&bool> = todo!("get used inputs from symbol table");

                let inputs = inputs
                    .iter_mut()
                    .zip(used_inputs)
                    .filter_map(|(expression, used)| {
                        if *used {
                            Some(expression.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                self.kind = StreamExpressionKind::UnitaryNodeApplication {
                    node_id: unitary_node_id,
                    inputs,
                    output_id,
                };
            }
            StreamExpressionKind::Expression { .. } => (),
            StreamExpressionKind::UnitaryNodeApplication { .. } => unreachable!(),
        }
    }
}
