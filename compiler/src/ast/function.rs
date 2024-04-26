use crate::ast::{expression::Expression, statement::Statement};
use crate::common::{location::Location, r#type::Type};

#[derive(Debug, PartialEq, serde::Serialize)]
/// LanGRust function AST.
pub struct Function {
    /// Function identifier.
    pub id: String,
    /// Function's inputs identifiers and their types.
    pub inputs: Vec<(String, Type)>,
    /// Function's statements.
    pub statements: Vec<Statement>,
    /// Function's returned expression and its type.
    pub returned: (Type, Expression),
    /// Function location.
    pub location: Location,
}
