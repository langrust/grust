/// A node's import.
#[derive(Debug, PartialEq)]
pub enum Import {
    /// Import of another node.
    NodeFile(String),
    /// Import of a function.
    Function(String),
}
