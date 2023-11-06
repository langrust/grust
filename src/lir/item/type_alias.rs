use crate::lir::r#type::Type;

/// Type alias in Rust.
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct TypeAlias {
    /// Visibility: `true` is public, `false` is private.
    pub public_visibility: bool,
    /// Alias for the type.
    pub name: String,
    /// Type that is aliased.
    pub r#type: Type,
}

impl std::fmt::Display for TypeAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let visibility = if self.public_visibility { "pub " } else { "" };
        write!(f, "{}type {} = {};", visibility, self.name, self.r#type)
    }
}

#[cfg(test)]
mod fmt {
    use crate::lir::{item::type_alias::TypeAlias, r#type::Type};

    #[test]
    fn should_format_type_alias_definition() {
        let alias = TypeAlias {
            public_visibility: true,
            name: String::from("Integer"),
            r#type: Type::Identifier {
                identifier: String::from("i64"),
            },
        };
        let control = String::from("pub type Integer = i64;");
        assert_eq!(format!("{}", alias), control)
    }
}
