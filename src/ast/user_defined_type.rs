use crate::util::{location::Location, type_system::Type};

#[derive(Debug, PartialEq)]
/// LanGRust user defined type AST.
pub enum UserDefinedType {
    /// Represents a structure definition.
    Structure {
        /// The structure's identifier.
        id: String,
        /// The structure's fields: a field has an identifier and a type.
        fields: Vec<(String, Type)>,
        /// Structure location.
        location: Location,
    },
    /// Represents an enumeration definition.
    Enumeration {
        /// The enumeration's identifier.
        id: String,
        /// The enumeration's elements.
        elements: Vec<String>,
        /// Enumeration location.
        location: Location,
    },
    /// Represents an array definition.
    Array {
        /// The array's identifier.
        id: String,
        /// The array's type.
        array_type: Type,
        /// The array's size.
        size: usize,
        /// Array location.
        location: Location,
    },
}
