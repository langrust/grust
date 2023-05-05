use crate::util::{location::Location, type_system::Type};

use super::expression::Expression;

#[derive(Debug, PartialEq, Clone)]
/// LanGRust calculus AST.
pub struct Calculus {
    /// Identifier of the new element.
    pub id: String,
    /// Element type.
    pub element_type: Type,
    /// The expression defining the element.
    pub expression: Expression,
    /// Calculus location.
    pub location: Location,
}
