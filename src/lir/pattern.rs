use crate::common::constant::Constant;

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust matching pattern LIR (resemble to the AST).
pub enum Pattern {
    /// Identifier pattern, gives a name to the matching expression.
    Identifier {
        /// Identifier.
        name: String,
    },
    /// Literal pattern, matches the given literal (constant).
    Literal {
        /// The matching literal (constant).
        literal: Constant,
    },
    /// Structure pattern that matches the structure and its fields.
    Structure {
        /// The structure id.
        name: String,
        /// The structure fields with the corresponding patterns to match.
        fields: Vec<(String, Pattern)>,
    },
    /// Enumeration pattern.
    Enumeration {
        /// The enumeration type name.
        enum_name: String,
        /// The element name.
        elem_name: String,
    },
    /// Tuple pattern that matches tuples.
    Tuple {
        /// The elements of the tuple.
        elements: Vec<Pattern>,
    },
    /// Some pattern that matches when an optional has a value which match the pattern.
    Some {
        /// The pattern matching the value.
        pattern: Box<Pattern>,
    },
    /// None pattern, matches when the optional does not have a value.
    None,
    /// The default pattern that matches anything.
    Default,
}
