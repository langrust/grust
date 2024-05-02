/// A node's import.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Import {
    /// Import of another node.
    NodeFile(String),
    /// Import of a function.
    Function(String),
    /// Import of an enumeration.
    Enumeration(String),
    /// Import of a structure.
    Structure(String),
    /// Import of an array alias.
    ArrayAlias(String),
    /// Import of creusot contracts
    Creusot(String),
}
