use crate::common::r#type::Type;

/// A structure definition.
pub struct Structure {
    /// The structure's name.
    pub name: String,
    /// The structure's fields.
    pub fields: Vec<(String, Type)>,
}
