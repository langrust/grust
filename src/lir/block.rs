use crate::lir::statement::Statement;

/// A block of statements.
pub struct Block {
    /// Statements of the block.
    pub statements: Vec<Statement>,
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let statements = self
            .statements
            .iter()
            .map(|statement| format!("{statement}"))
            .collect::<Vec<_>>()
            .join(" ");
        write!(f, "{{{statements}}}")
    }
}
