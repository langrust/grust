use crate::ast::{constant::Constant, location::Location};

#[derive(Debug, PartialEq, Clone)]
/// LanGRust matching pattern AST.
pub enum Pattern {
    /// Identifier pattern, gives a name to the matching expression.
    Identifier {
        /// Identifier name.
        name: String,
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
