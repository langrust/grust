use crate::lir::r#type::Type;

/// Type alias in Rust.
pub struct TypeAlias {
    /// Visibility: `true` is public, `false` is private.
    pub public_visibility: bool,
    /// Alias for the type.
    pub name: String,
    /// Type that is aliased.
    pub r#type: Type,
}

impl std::fmt::Display for TypeAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let visibility = if self.public_visibility { "pub " } else { "" };
        write!(f, "{}type {} = {};", visibility, self.name, self.r#type)
    }
}
