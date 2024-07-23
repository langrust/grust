//! LIR [Enumeration] module.

prelude! {}

/// An enumeration definition.
#[derive(Debug, PartialEq)]
pub struct Enumeration {
    /// The enumeration's name.
    pub name: String,
    /// The enumeration's elements.
    pub elements: Vec<String>,
}

mk_new! { impl Enumeration =>
    new {
        name: impl Into<String> = name.into(),
        elements: Vec<String>
    }
}
