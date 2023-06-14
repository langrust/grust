use crate::common::{location::Location, type_system::Type};
use crate::ir::expression::Expression;

#[derive(Debug, PartialEq, Clone)]
/// LanGRust statement AST.
pub struct Statement {
    /// Identifier of the new element.
    pub id: String,
    /// Element type.
    pub element_type: Type,
    /// The expression defining the element.
    pub expression: Expression,
    /// Statement location.
    pub location: Location,
}
