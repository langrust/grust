use crate::common::r#type::Type;

/// An array alias definition.
#[derive(Debug, PartialEq)]
pub struct ArrayAlias {
    /// The array's name.
    pub name: String,
    /// The array's type.
    pub array_type: Type,
    /// The array's size.
    pub size: usize,
}
