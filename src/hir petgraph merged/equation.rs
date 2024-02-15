use crate::common::location::Location;
use crate::hir::stream_expression::StreamExpression;

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust equation HIR.
pub struct Equation {
    /// Identifier of the signal.
    pub id: usize,
    /// The stream expression defining the signal.
    pub expression: StreamExpression,
    /// Equation location.
    pub location: Location,
}
