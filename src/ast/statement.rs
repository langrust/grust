use crate::ast::expression::Expression;
use crate::common::{location::Location, r#type::Type};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
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
