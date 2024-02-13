use std::collections::HashMap;

use crate::ast::typedef::Typedef;
use crate::common::{constant::Constant, context::Context, location::Location, r#type::Type};
use crate::error::{Error, TerminationError};

use std::fmt::{self, Display};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust matching pattern AST.
pub enum Pattern {
    /// Identifier pattern, gives a name to the matching expression.
    Identifier {
        /// Identifier.
        id: usize,
        /// Pattern location.
        location: Location,
    },
    /// Constant pattern, matches le given constant.
    Constant {
        /// The matching constant.
        constant: Constant,
        /// Pattern location.
        location: Location,
    },
    /// Structure pattern that matches the structure and its fields.
    Structure {
        /// The structure name.
        name: String,
        /// The structure fields with the corresponding patterns to match.
        fields: Vec<(String, Pattern)>,
        /// Pattern location.
        location: Location,
    },
    /// Tuple pattern that matches tuples.
    Tuple {
        /// The elements of the tuple.
        elements: Vec<Pattern>,
        /// Pattern location.
        location: Location,
    },
    /// Some pattern that matches when an optional has a value which match the pattern.
    Some {
        /// The pattern matching the value.
        pattern: Box<Pattern>,
        /// Pattern location.
        location: Location,
    },
    /// None pattern, matches when the optional does not have a value.
    None {
        /// Pattern location.
        location: Location,
    },
    /// The default pattern that matches anything.
    Default {
        /// Pattern location.
        location: Location,
    },
}
