use crate::common::location::Location;
use crate::hir::stream_expression::StreamExpression;

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust statement HIR.
pub struct Statement<E> {
    /// Identifier of the element.
    pub id: usize,
    /// The expression defining the element.
    pub expression: E,
    /// Statement location.
    pub location: Location,
}

impl Statement<StreamExpression> {
    pub fn no_fby(&self) -> bool {
        self.expression.no_fby()
    }
    pub fn is_normal_form(&self) -> bool {
        self.expression.is_normal_form()
    }
    pub fn no_node_application(&self) -> bool {
        self.expression.no_node_application()
    }
}
