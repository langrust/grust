use crate::{
    common::r#type::Type,
    lir::{block::Block, item::import::Import},
};

/// A function definition.
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct Function {
    /// The function's name.
    pub name: String,
    /// The inputs.
    pub inputs: Vec<(String, Type)>,
    /// The output type.
    pub output: Type,
    /// The body of the function.
    pub body: Block,
    /// The imports (used typedefs).
    pub imports: Vec<Import>,
}
