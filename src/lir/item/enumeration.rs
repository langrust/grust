/// A Rust enumeration.
pub struct Enumeration {
    /// Visibility: `true` is public, `false` is private.
    pub public_visibility: bool,
    /// Name of the enumeration.
    pub name: String,
    /// All the elements of the enumeration.
    pub elements: Vec<String>,
}

impl std::fmt::Display for Enumeration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let visibility = if self.public_visibility { "pub " } else { "" };
        let elements = self.elements.join(", ");
        write!(f, "{}enum {} {{{}}}", visibility, self.name, elements)
    }
}
