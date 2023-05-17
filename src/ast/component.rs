use crate::ast::{location::Location, type_system::Type};

use crate::ast::equation::Equation;

#[derive(Debug, PartialEq)]
/// LanGRust component AST.
pub struct Component {
    /// Component identifier.
    pub id: String,
    /// Component's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Component's equations.
    pub equations: Vec<(String, Equation)>,
    /// Component location.
    pub location: Location,
}
