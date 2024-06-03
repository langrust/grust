prelude! {}

/// A block declaration.
#[derive(Debug, PartialEq)]
pub struct Block {
    /// The block's statements.
    pub statements: Vec<Stmt>,
}

impl Block {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self { statements }
    }
}
