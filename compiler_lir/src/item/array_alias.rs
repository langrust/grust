//! LIR [ArrayAlias] module.

prelude! {}

/// An array alias definition.
#[derive(Debug, PartialEq)]
pub struct ArrayAlias {
    /// The array's name.
    pub name: String,
    /// The array's type.
    pub array_type: Typ,
    /// The array's size.
    pub size: usize,
}

mk_new! { impl ArrayAlias =>
    new {
        name: impl Into<String> = name.into(),
        array_type: Typ,
        size: usize,
    }
}
