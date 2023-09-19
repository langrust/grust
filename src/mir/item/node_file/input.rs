use crate::common::r#type::Type;

/// A node input structure.
pub struct Input {
    /// The node's name.
    pub node_name: String,
    /// The input's elements.
    pub elements: Vec<InputElement>,
}

/// An input element structure.
pub struct InputElement {
    /// The name of the input.
    pub identifier: String,
    /// The type of the input.
    pub r#type: Type,
}
