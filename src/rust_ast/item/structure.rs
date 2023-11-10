use crate::rust_ast::r#type::Type;

/// A Rust structure.
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct Structure {
    /// Visibility: `true` is public, `false` is private.
    pub public_visibility: bool,
    /// Name of the structure.
    pub name: String,
    /// All the elements of the structure.
    pub fields: Vec<Field>,
}

impl std::fmt::Display for Structure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let visibility = if self.public_visibility { "pub " } else { "" };
        let fields = self
            .fields
            .iter()
            .map(|field| format!("{field}"))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{}struct {} {{ {} }}", visibility, self.name, fields)
    }
}

/// A structure's field.
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct Field {
    /// Visibility: `true` is public, `false` is private.
    pub public_visibility: bool,
    /// Name of the field.
    pub name: String,
    /// Type of the field.
    pub r#type: Type,
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let visibility = if self.public_visibility { "pub " } else { "" };
        write!(f, "{}{}: {}", visibility, self.name, self.r#type)
    }
}

#[cfg(test)]
mod fmt {
    use crate::rust_ast::{
        item::structure::{Field, Structure},
        r#type::Type,
    };

    #[test]
    fn should_format_structure_definition() {
        let structure = Structure {
            public_visibility: true,
            name: String::from("Point"),
            fields: vec![
                Field {
                    public_visibility: true,
                    name: String::from("x"),
                    r#type: Type::Identifier {
                        identifier: String::from("i64"),
                    },
                },
                Field {
                    public_visibility: true,
                    name: String::from("y"),
                    r#type: Type::Identifier {
                        identifier: String::from("i64"),
                    },
                },
                Field {
                    public_visibility: false,
                    name: String::from("z"),
                    r#type: Type::Identifier {
                        identifier: String::from("i64"),
                    },
                },
            ],
        };
        let control = String::from("pub struct Point { pub x: i64, pub y: i64, z: i64 }");
        assert_eq!(format!("{}", structure), control)
    }
}
