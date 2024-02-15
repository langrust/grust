use crate::common::{constant::Constant, location::Location, r#type::Type};

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
/// LanGRust matching pattern HIR.
pub enum PatternKind {
    /// Identifier pattern, gives a name to the matching expression.
    Identifier {
        /// Identifier.
        id: usize,
    },
    /// Constant pattern, matches le given constant.
    Constant {
        /// The matching constant.
        constant: Constant,
    },
    /// Structure pattern that matches the structure and its fields.
    Structure {
        /// The structure id.
        id: usize,
        /// The structure fields with the corresponding patterns to match.
        fields: Vec<(usize, Pattern)>,
    },
    /// Enumeration pattern.
    Enumeration {
        /// The enumeration type id.
        enum_id: usize,
        /// The element id.
        elem_id: usize,
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

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct Pattern {
    /// Pattern kind.
    pub kind: PatternKind,
    /// Pattern type.
    pub typing: Option<Type>,
    /// Pattern location.
    pub location: Location,
}
impl Pattern {
    pub fn get_type(&self) -> Option<&Type> {
        self.typing.as_ref()
    }
    pub fn get_type_mut(&mut self) -> Option<&mut Type> {
        self.typing.as_mut()
    }
}
