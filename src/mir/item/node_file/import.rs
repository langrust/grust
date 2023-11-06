/// A node's import.
#[derive(Debug, PartialEq, serde::Serialize)]
pub enum Import {
    /// Import of another node.
    NodeFile(String),
    /// Import of a function.
    Function(String),
}
