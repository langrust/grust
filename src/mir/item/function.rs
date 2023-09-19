use crate::{common::r#type::Type, mir::block::Block};

/// A function definition.
pub struct Function {
    /// The function's name.
    pub name: String,
    /// The inputs.
    pub inputs: Vec<(String, Type)>,
    /// The output type.
    pub output: Type,
    /// The body of the function.
    pub body: Block,
}
