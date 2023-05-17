use crate::ast::{location::Location, type_system::Type, scope::Scope};

use super::stream_expression::StreamExpression;

#[derive(Debug, PartialEq, Clone)]
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
