use crate::common::{constant::Constant, r#type::Type};

#[derive(Debug, PartialEq, Clone)]
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
    /// Typed pattern.
    Typed {
        /// The pattern.
        pattern: Box<Pattern>,
        /// The type.
        typing: Type,
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
        /// The optional element of the enumeration.
        element: Option<Box<Pattern>>,
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

impl Pattern {
    pub fn ident(name: impl Into<String>) -> Self {
        Self::Identifier { name: name.into() }
    }
    pub fn literal(literal: Constant) -> Self {
        Self::Literal { literal }
    }
    pub fn typed(pat: Pattern, typing: Type) -> Self {
        Self::Typed {
            pattern: Box::new(pat),
            typing,
        }
    }
    pub fn structure(name: impl Into<String>, fields: Vec<(String, Pattern)>) -> Self {
        Self::Structure {
            name: name.into(),
            fields,
        }
    }
    pub fn enumeration(
        enum_name: impl Into<String>,
        elem_name: impl Into<String>,
        element: Option<Self>,
    ) -> Self {
        Self::Enumeration {
            enum_name: enum_name.into(),
            elem_name: elem_name.into(),
            element: element.map(Box::new),
        }
    }
    pub fn tuple(elements: Vec<Self>) -> Self {
        Self::Tuple { elements }
    }
    pub fn some(pat: Self) -> Self {
        Self::Some {
            pattern: Box::new(pat),
        }
    }
    pub fn none() -> Self {
        Self::None
    }
    pub fn default() -> Self {
        Self::Default
    }
}
