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

/// A structure's field.
pub struct Field {
    /// Visibility: `true` is public, `false` is private.
    pub public_visibility: bool,
    /// Name of the field.
    pub name: String,
    /// Type of the field.
    pub r#type: Type,
}
