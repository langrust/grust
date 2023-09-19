use crate::mir::statement::Statement;

/// A block declaration.
pub struct Block {
    /// The block's statements.
    pub statements: Vec<Statement>,
}
