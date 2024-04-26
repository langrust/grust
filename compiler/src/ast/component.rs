use crate::ast::equation::Equation;
use crate::common::{location::Location, r#type::Type};

use super::contract::Contract;

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust component AST.
pub struct Component {
    /// Component identifier.
    pub id: String,
    /// Is true when the component is a service.
    pub is_service: bool,
    /// Component's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Component's period of execution.
    pub period: Option<usize>,
    /// Component's equations.
    pub equations: Vec<(String, Equation)>,
    /// Component's contract.
    pub contract: Contract,
    /// Component location.
    pub location: Location,
}
