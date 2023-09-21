/// A Rust enumeration.
#[derive(Debug, PartialEq)]
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
        write!(f, "{}enum {} {{ {} }}", visibility, self.name, elements)
    }
}

#[cfg(test)]
mod fmt {
    use super::Enumeration;

    #[test]
    fn should_format_enumeration_definition() {
        let enumeration = Enumeration {
            public_visibility: true,
            name: String::from("Color"),
            elements: vec![
                String::from("Blue"),
                String::from("Red"),
                String::from("Yellow"),
                String::from("Green"),
                String::from("Purple"),
            ],
        };
        let control = String::from("pub enum Color { Blue, Red, Yellow, Green, Purple }");
        assert_eq!(format!("{}", enumeration), control)
    }
}
