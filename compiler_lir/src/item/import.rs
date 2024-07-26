use ast::common::syn;

/// A state-machine import structure.
#[derive(Debug, PartialEq)]
pub struct Import {
    /// The node's name.
    pub name: String,
    /// The path of the import.
    pub path: syn::Path,
}
