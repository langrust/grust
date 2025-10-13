use crate::common::location::Location;
use crate::hir::{expression::Expression, statement::Statement};

#[derive(Debug, PartialEq, serde::Serialize)]
/// LanGRust function AST.
pub struct Function {
    /// Function identifier.
    pub id: usize,
    /// Function's statements.
    pub statements: Vec<Statement<Expression>>,
    /// Function's returned expression and its type.
    pub returned: Expression,
    /// Function location.
    pub location: Location,
}
