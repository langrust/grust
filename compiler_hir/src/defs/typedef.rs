//! HIR [Typedef] module.

prelude! {}

#[derive(Debug, PartialEq, Clone)]
/// LanGRust user defined type AST.
pub enum Kind {
    /// Represents a structure definition.
    Structure {
        /// The structure's fields: a field has an identifier and a type.
        fields: Vec<usize>,
    },
    /// Represents an enumeration definition.
    Enumeration {
        /// The enumeration's elements.
        elements: Vec<usize>,
    },
    /// Represents an array definition.
    Array,
}

#[derive(Debug, PartialEq, Clone)]
/// LanGRust user defined type AST.
pub struct Typedef {
    /// Typedef identifier.
    pub id: usize,
    /// Typedef kind.
    pub kind: Kind,
    /// Typedef location.
    pub loc: Location,
}
