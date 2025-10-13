use super::item::Item;

#[derive(serde::Serialize)]
/// A project structure.
pub struct Project {
    /// The project's items.
    pub items: Vec<Item>,
}
