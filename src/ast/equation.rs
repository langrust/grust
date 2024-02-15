use crate::ast::stream_expression::StreamExpression;
use crate::common::{location::Location, r#type::Type, scope::Scope};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust equation AST.
pub struct Equation {
    /// Signal's scope.
    pub scope: Scope,
    /// Identifier of the signal.
    pub id: String,
    /// Signal type.
    pub signal_type: Type,
    /// The stream expression defining the signal.
    pub expression: StreamExpression,
    /// Equation location.
    pub location: Location,
}
