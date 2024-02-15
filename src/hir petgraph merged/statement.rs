use crate::common::location::Location;
use crate::hir::expression::Expression;

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust statement HIR.
pub struct Statement {
    /// Identifier of the element.
    pub id: usize,
    /// The expression defining the element.
    pub expression: Expression,
    /// Statement location.
    pub location: Location,
}
