//! LIR [Input] module.

prelude! {}

/// A node input structure.
#[derive(Debug, PartialEq)]
pub struct Input {
    /// The node's name.
    pub node_name: String,
    /// The input's elements.
    pub elements: Vec<InputElement>,
}

/// An input element structure.
#[derive(Debug, PartialEq)]
pub struct InputElement {
    /// The name of the input.
    pub identifier: String,
    /// The type of the input.
    pub typ: Typ,
}
