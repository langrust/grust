use std::collections::HashMap;

use crate::common::context::Context;
use crate::common::{location::Location, r#type::Type};
use crate::error::{Error, TerminationError};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust user defined type AST.
pub enum Typedef {
    /// Represents a structure definition.
    Structure {
        /// The structure's identifier.
        id: usize,
        /// The structure's fields: a field has an identifier and a type.
        fields: Vec<usize>,
        /// Structure location.
        location: Location,
    },
    /// Represents an enumeration definition.
    Enumeration {
        /// The enumeration's identifier.
        id: usize,
        /// The enumeration's elements.
        elements: Vec<usize>,
        /// Enumeration location.
        location: Location,
    },
    /// Represents an array definition.
    Array {
        /// The array's identifier.
        id: usize,
        /// The array's type.
        array_type: Type,
        /// The array's size.
        size: usize,
        /// Array location.
        location: Location,
    },
}
