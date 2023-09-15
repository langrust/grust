use super::item::Item;

/// HIR of a Rust source code file.
pub struct File {
    /// File's path.
    pub path: String,
    /// Items present in the file.
    pub items: Vec<Item>,
}

impl std::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let items = self
            .items
            .iter()
            .map(|item| format!("{item}"))
            .collect::<Vec<_>>()
            .join(" ");
        write!(f, "{items}")
    }
}
