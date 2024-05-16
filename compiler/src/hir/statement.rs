use crate::common::location::Location;
use crate::hir::{stream_expression::StreamExpression, pattern::Pattern};

#[derive(Debug, PartialEq, Clone)]
/// LanGRust statement HIR.
pub struct Statement<E> {
    /// Pattern of elements.
    pub typed_pattern: Pattern,
    /// The expression defining the element.
    pub expression: E,
    /// Statement location.
    pub location: Location,
}

impl Statement<StreamExpression> {
    /// Tell if there is no FBY expression.
    pub fn no_fby(&self) -> bool {
        self.expression.no_fby()
    }
    /// Tell if it is in normal form.
    pub fn is_normal_form(&self) -> bool {
        self.expression.is_normal_form()
    }
    /// Tell if there is no node application.
    pub fn no_node_application(&self) -> bool {
        self.expression.no_node_application()
    }
}
