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
