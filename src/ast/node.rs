use crate::ast::equation::Equation;
use crate::common::{location::Location, r#type::Type};

use super::contract::Contract;

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
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
    /// Node's contract.
    pub contract: Contract,
    /// Node location.
    pub location: Location,
}
