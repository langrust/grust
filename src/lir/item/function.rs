use crate::lir::{block::Block, item::signature::Signature};

/// A function definition in Rust.
pub struct Function {
    /// Function's signature.
    pub signature: Signature,
    /// Function's body.
    pub body: Block,
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.signature, self.body)
    }
}
