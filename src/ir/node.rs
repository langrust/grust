use crate::common::{location::Location, type_system::Type};
use crate::ir::equation::Equation;

#[derive(Debug, PartialEq, Clone)]
/// LanGRust node AST.
pub struct Node {
    /// Node identifier.
    pub id: String,
    /// Is true when the node is a component.
    pub is_component: bool,
    /// Node's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Node's equations.
    pub equations: Vec<(String, Equation)>,
    /// Node location.
    pub location: Location,
}
