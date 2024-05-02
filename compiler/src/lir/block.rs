use crate::lir::statement::Statement;

/// A block declaration.
#[derive(Debug, PartialEq)]
pub struct Block {
    /// The block's statements.
    pub statements: Vec<Statement>,
}
