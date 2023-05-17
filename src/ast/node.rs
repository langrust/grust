use crate::ast::{equation::Equation, location::Location, type_system::Type};

#[derive(Debug, PartialEq)]
/// LanGRust node AST.
pub struct Node {
    /// Node identifier.
    pub id: String,
    /// Node's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Node's equations.
    pub equations: Vec<(String, Equation)>,
    /// Node location.
    pub location: Location,
}
