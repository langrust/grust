use crate::util::{location::Location, type_system::Type};

use crate::ast::{calculus::Calculus, expression::Expression};

#[derive(Debug, PartialEq)]
/// LanGRust function AST.
pub struct Function {
    /// Function identifier.
    pub id: String,
    /// Function's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Function's calculi.
    pub calculi: Vec<(String, Calculus)>,
    /// Function's returned expression and its type.
    pub returned: (Type, Expression),
    /// Function location.
    pub location: Location,
}
