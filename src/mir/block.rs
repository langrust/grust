use crate::mir::statement::Statement;

/// A block declaration.
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct Block {
    /// The block's statements.
    pub statements: Vec<Statement>,
}
