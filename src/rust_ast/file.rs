use super::item::Item;

#[derive(serde::Serialize)]
/// HIR of a Rust source code file.
pub struct File {
    /// File's path.
    pub path: String,
    /// Items present in the file.
    pub items: Vec<Item>,
}
impl File {
    /// Create a new file.
    pub fn new(path: String) -> Self {
        File {
            path,
            items: vec![],
        }
    }

    /// Add item.
    pub fn add_item(&mut self, item: Item) {
        self.items.push(item)
    }

    /// Generate the file at its location path.
    pub fn generate(&self) {
        let file_str = self.to_string();
        let syntax_tree: syn::File = syn::parse_str(&file_str).unwrap();
        let pretty_file = prettyplease::unparse(&syntax_tree);

        if let Some(p) = AsRef::<std::path::Path>::as_ref(&self.path).parent() {
            std::fs::create_dir_all(p).unwrap()
        };
        std::fs::write(self.path.clone(), pretty_file).unwrap();
    }
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
