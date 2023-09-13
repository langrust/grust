use super::item::Item;

/// HIR of a Rust source code file.
pub struct File {
    /// File's path.
    pub path: String,
    /// Items present in the file.
    pub items: Vec<Item>,
}
