use crate::common::{constant::Constant, location::Location};

#[derive(Debug, PartialEq, Clone)]
/// GRust matching pattern AST.
pub enum PatternKind {
    /// Identifier pattern, gives a name to the matching expression.
    Identifier {
        /// Identifier name.
        name: String,
    },
    /// Constant pattern, matches le given constant.
    Constant {
        /// The matching constant.
        constant: Constant,
    },
    /// Structure pattern that matches the structure and its fields.
    Structure {
        /// The structure name.
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

#[derive(Debug, PartialEq, Clone)]
/// GRust matching pattern AST.
pub struct Pattern {
    /// Pattern kind.
    pub kind: PatternKind,
    /// Pattern location.
    pub location: Location,
}
