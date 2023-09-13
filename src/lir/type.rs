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
