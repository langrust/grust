prelude! { Block }

/// A function definition.
#[derive(Debug, PartialEq)]
pub struct Function {
    /// The function's name.
    pub name: String,
    /// The inputs.
    pub inputs: Vec<(String, Typ)>,
    /// The output type.
    pub output: Typ,
    /// The body of the function.
    pub body: Block,
    /// The contract to prove.
    pub contract: Contract,
}
