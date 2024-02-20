use crate::{
    hir::{expression::ExpressionKind, stream_expression::StreamExpression},
    symbol_table::SymbolTable,
};

impl ExpressionKind<StreamExpression> {
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
        match self {
            ExpressionKind::Constant { .. }
            | ExpressionKind::Identifier { .. }
            | ExpressionKind::Abstraction { .. }
            | ExpressionKind::Enumeration { .. } => (),
            ExpressionKind::Application {
                function_expression,
                inputs,
            } => {
                function_expression
                    .change_node_application_into_unitary_node_application(symbol_table);
                inputs.iter_mut().for_each(|expression| {
                    expression.change_node_application_into_unitary_node_application(symbol_table)
                })
            }
            ExpressionKind::Structure { fields, .. } => {
                fields.iter_mut().for_each(|(_, expression)| {
                    expression.change_node_application_into_unitary_node_application(symbol_table)
                })
            }
            ExpressionKind::Array { elements } => elements.iter_mut().for_each(|expression| {
                expression.change_node_application_into_unitary_node_application(symbol_table)
            }),
            ExpressionKind::Match { expression, arms } => {
                expression.change_node_application_into_unitary_node_application(symbol_table);
                arms.iter_mut().for_each(|(_, option, block, expression)| {
                    debug_assert!(block.is_empty());
                    option.as_mut().map(|expression| {
                        expression
                            .change_node_application_into_unitary_node_application(symbol_table)
                    });
                    expression.change_node_application_into_unitary_node_application(symbol_table)
                })
            }
            ExpressionKind::When {
                option,
                present,
                present_body,
                default,
                default_body,
                ..
            } => {
                debug_assert!(present_body.is_empty());
                debug_assert!(default_body.is_empty());
                option.change_node_application_into_unitary_node_application(symbol_table);
                present.change_node_application_into_unitary_node_application(symbol_table);
                default.change_node_application_into_unitary_node_application(symbol_table);
            }
            ExpressionKind::FieldAccess { expression, .. } => {
                expression.change_node_application_into_unitary_node_application(symbol_table)
            }
            ExpressionKind::TupleElementAccess { expression, .. } => {
                expression.change_node_application_into_unitary_node_application(symbol_table)
            }
            ExpressionKind::Map {
                expression,
                function_expression,
            } => {
                expression.change_node_application_into_unitary_node_application(symbol_table);
                function_expression
                    .change_node_application_into_unitary_node_application(symbol_table)
            }
            ExpressionKind::Fold {
                expression,
                initialization_expression,
                function_expression,
            } => {
                expression.change_node_application_into_unitary_node_application(symbol_table);
                initialization_expression
                    .change_node_application_into_unitary_node_application(symbol_table);
                function_expression
                    .change_node_application_into_unitary_node_application(symbol_table)
            }
            ExpressionKind::Sort {
                expression,
                function_expression,
            } => {
                expression.change_node_application_into_unitary_node_application(symbol_table);
                function_expression
                    .change_node_application_into_unitary_node_application(symbol_table)
            }
            ExpressionKind::Zip { arrays } => arrays.iter_mut().for_each(|expression| {
                expression.change_node_application_into_unitary_node_application(symbol_table)
            }),
        }
    }
}
