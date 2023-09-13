/// A Rust enumeration.
pub struct Enumeration {
    /// Visibility: `true` is public, `false` is private.
    pub public_visibility: bool,
    /// Name of the enumeration.
    pub name: String,
    /// All the elements of the enumeration.
    pub elements: Vec<String>,
}
