use crate::common::{location::Location, r#type::Type};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust user defined type AST.
pub enum TypedefKind {
    /// Represents a structure definition.
    Structure {
        /// The structure's fields: a field has an identifier and a type.
        fields: Vec<(String, Type)>,
    },
    /// Represents an enumeration definition.
    Enumeration {
        /// The enumeration's elements.
        elements: Vec<String>,
    },
    /// Represents an array definition.
    Array {
        /// The array's type.
        array_type: Type,
        /// The array's size.
        size: usize,
    },
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust user defined type AST.
pub struct Typedef {
    /// Typedef identifier.
    pub id: String,
    /// Typedef kind.
    pub kind: TypedefKind,
    /// Typedef location.
    pub location: Location,
}
