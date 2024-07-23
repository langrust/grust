//! LIR [Pattern] module.

prelude! {}

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
        typing: Typ,
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
    /// Ok pattern that matches when a result has a value which match the pattern.
    Ok {
        /// The pattern matching the value.
        pattern: Box<Pattern>,
    },
    /// Err pattern, matches when the result does not have a value.
    Err,
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

mk_new! { impl Pattern =>
    Identifier: ident { name: impl Into<String> = name.into() }
    Literal: literal {literal: Constant }
    Typed: typed {
        pattern: Self = Box::new(pattern),
        typing: Typ
    }
    Structure: structure {
        name: impl Into<String> = name.into(),
        fields: Vec<(String, Self)>
    }
    Enumeration: enumeration {
        enum_name: impl Into<String> = enum_name.into(),
        elem_name: impl Into<String> = elem_name.into(),
        element: Option<Self> = element.map(Box::new),
    }
    Tuple: tuple { elements: Vec<Self> }
    Ok: ok { pattern: Self = Box::new(pattern) }
    Err: err()
    Some: some { pattern: Self = Box::new(pattern) }
    None: none()
    Default: default()
}
