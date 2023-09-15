use crate::lir::statement::Statement;

/// A block of statements.
pub struct Block {
    /// Statements of the block.
    pub statements: Vec<Statement>,
}
