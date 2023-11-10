/// An enumeration definition.
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct Enumeration {
    /// The enumeration's name.
    pub name: String,
    /// The enumeration's elements.
    pub elements: Vec<String>,
}
