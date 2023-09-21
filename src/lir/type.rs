use crate::common::r#type::Type as DSLType;

/// The four different kind of type in Rust.
#[derive(Debug, PartialEq)]
pub enum Type {
    /// Simple type.
    Identifier {
        /// The identifier to the type.
        identifier: String,
    },
    /// Array type.
    Array {
        /// Type of the elements.
        element: Box<Type>,
        /// Size of the array.
        size: usize,
    },
    /// Function type.
    Function {
        /// Types of the arguments.
        arguments: Vec<Type>,
        /// Output type.
        output: Box<Type>,
    },
    /// Closure type.
    Closure {
        /// Types of the arguments.
        arguments: Vec<Type>,
        /// Output type.
        output: Box<Type>,
    },
    /// Realization of generic type.
    Generic {
        /// The generic type.
        generic: Box<Type>,
        /// The ralization arguments.
        arguments: Vec<Type>,
    },
    /// Reference type.
    Reference {
        /// Optionally mutable.
        mutable: bool,
        /// The type of the element in reference.
        element: Box<Type>,
    },
    /// Tuple type.
    Tuple {
        /// Ordered types in the tuple.
        elements: Vec<Type>,
    },
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Owned(r#type) => write!(f, "{}", r#type),
            Type::Mutable(r#type) => write!(f, "mut {}", r#type),
            Type::Reference(r#type) => write!(f, "&{}", r#type),
            Type::MutableReference(r#type) => write!(f, "&mut {}", r#type),
        }
    }
}

#[cfg(test)]
mod fmt {
    use super::Type;
    use crate::common::r#type::Type as DSLType;

    #[test]
    fn should_format_rust_owned_type() {
        let r#type = Type::Owned(DSLType::Integer);
        let control = String::from("i64");
        assert_eq!(format!("{}", r#type), control)
    }

    #[test]
    fn should_format_rust_mutable_type() {
        let r#type = Type::Mutable(DSLType::Integer);
        let control = String::from("mut i64");
        assert_eq!(format!("{}", r#type), control)
    }

    #[test]
    fn should_format_rust_reference_type() {
        let r#type = Type::Reference(DSLType::Integer);
        let control = String::from("&i64");
        assert_eq!(format!("{}", r#type), control)
    }

    #[test]
    fn should_format_rust_mutable_reference_type() {
        let r#type = Type::MutableReference(DSLType::Integer);
        let control = String::from("&mut i64");
        assert_eq!(format!("{}", r#type), control)
    }
}
