use crate::lir::r#type::Type;

/// A Rust structure.
pub struct Structure {
    /// Visibility: `true` is public, `false` is private.
    pub public_visibility: bool,
    /// Name of the structure.
    pub name: String,
    /// All the elements of the structure.
    pub fields: Vec<Field>,
}

impl std::fmt::Display for Structure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let visibility = if self.public_visibility { "pub " } else { "" };
        let fields = self
            .fields
            .iter()
            .map(|field| format!("{field}"))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{}struct {} {{{}}}", visibility, self.name, fields)
    }
}

/// A structure's field.
pub struct Field {
    /// Visibility: `true` is public, `false` is private.
    pub public_visibility: bool,
    /// Name of the field.
    pub name: String,
    /// Type of the field.
    pub r#type: Type,
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let visibility = if self.public_visibility { "pub " } else { "" };
        write!(f, "{}{}: {}", visibility, self.name, self.r#type)
    }
}
