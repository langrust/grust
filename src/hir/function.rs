use std::collections::HashMap;

use crate::ast::typedef::Typedef;
use crate::common::{context::Context, location::Location, r#type::Type};
use crate::error::{Error, TerminationError};
use crate::hir::{expression::Expression, statement::Statement};

#[derive(Debug, PartialEq, serde::Serialize)]
/// LanGRust function AST.
pub struct Function {
    /// Function identifier.
    pub id: usize,
    /// Function's inputs identifiers and their types.
    pub inputs: Vec<usize>,
    /// Function's statements.
    pub statements: Vec<Statement>,
    /// Function's returned expression and its type.
    pub returned: (Type, Expression),
    /// Function location.
    pub location: Location,
}
