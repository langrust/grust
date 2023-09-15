use crate::common::r#type::Type as DSLType;

/// The four different kind of type in Rust.
pub enum Type {
    /// Owned type.
    Owned(DSLType),
    /// Mutable owned type.
    Mutable(DSLType),
    /// Reference type.
    Reference(DSLType),
    /// Mutable reference type.
    MutableReference(DSLType),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Owned(r#type) => write!(f, "{}", r#type),
            Type::Mutable(r#type) => write!(f, "mut {}", r#type),
            Type::Reference(r#type) => write!(f, "&{}", r#type),
            Type::MutableReference(r#type) => write!(f, "&mut {}", r#type),
        }
    }
}
