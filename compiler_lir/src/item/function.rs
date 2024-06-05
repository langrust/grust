prelude! { Block, item::Import }

/// A function definition.
#[derive(Debug, PartialEq)]
pub struct Function {
    /// The function's name.
    pub name: String,
    /// The input's generic types.
    pub generics: Vec<(String, Typ)>,
    /// The inputs.
    pub inputs: Vec<(String, Typ)>,
    /// The output type.
    pub output: Typ,
    /// The body of the function.
    pub body: Block,
    /// The imports (used typedefs).
    pub imports: Vec<Import>,
    /// The contract to prove.
    pub contract: Contract,
}
