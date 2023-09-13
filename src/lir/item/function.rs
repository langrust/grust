use crate::lir::{block::Block, item::signature::Signature};

/// A function definition in Rust.
pub struct Function {
    /// Function's signature.
    pub signature: Signature,
    /// Function's body.
    pub body: Block,
}
