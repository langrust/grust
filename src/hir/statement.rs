use crate::common::location::Location;

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
