/// A node's import.
pub enum Import {
    /// Import of another node.
    NodeFile(String),
    /// Import of a function.
    Function(String),
}
