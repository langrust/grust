use std::collections::HashMap;

use crate::ast::{expression::Expression, typedef::Typedef};
use crate::common::{location::Location, r#type::Type};
use crate::error::{Error, TerminationError};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust statement AST.
pub struct Statement {
    /// Identifier of the element.
    pub id: usize,
    /// The expression defining the element.
    pub expression: Expression,
    /// Statement location.
    pub location: Location,
}
