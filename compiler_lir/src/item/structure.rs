prelude! {}

/// A structure definition.
#[derive(Debug, PartialEq)]
pub struct Structure {
    /// The structure's name.
    pub name: String,
    /// The structure's fields.
    pub fields: Vec<(String, Typ)>,
}

mk_new! { impl Structure =>
    new {
        name: impl Into<String> = name.into(),
        fields: Vec<(String, Typ)>,
    }
}
