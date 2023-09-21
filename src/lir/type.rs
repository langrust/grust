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
            Type::Identifier { identifier } => write!(f, "{}", identifier),
            Type::Array { element, size } => write!(f, "[{}; {}]", *element, size),
            Type::Function { arguments, output } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| format!("{argument}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                match output.as_ref() {
                    Type::Identifier { identifier } if identifier == &String::from("()") => {
                        write!(f, "fn({})", arguments)
                    }
                    _ => write!(f, "fn({}) -> {}", arguments, *output),
                }
            }
            Type::Closure { arguments, output } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| format!("{argument}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                match output.as_ref() {
                    Type::Identifier { identifier } if identifier == &String::from("()") => {
                        write!(f, "impl Fn({})", arguments)
                    }
                    _ => write!(f, "impl Fn({}) -> {}", arguments, *output),
                }
            }
            Type::Generic { generic, arguments } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| format!("{argument}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{}<{}>", generic, arguments)
            }
            Type::Reference { mutable, element } => {
                let mutable = if *mutable { "mut " } else { "" };
                write!(f, "&{}{}", mutable, *element)
            }
            Type::Tuple { elements } => {
                let elements = elements
                    .iter()
                    .map(|element| format!("{element}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "({})", elements)
            }
        }
    }
}

#[cfg(test)]
mod fmt {
    use super::Type;

    #[test]
    fn should_format_rust_simple_type() {
        let r#type = Type::Identifier {
            identifier: String::from("i64"),
        };
        let control = String::from("i64");
        assert_eq!(format!("{}", r#type), control)
    }

    #[test]
    fn should_format_rust_array_type() {
        let r#type = Type::Array {
            element: Box::new(Type::Identifier {
                identifier: String::from("i64"),
            }),
            size: 5,
        };
        let control = String::from("[i64; 5]");
        assert_eq!(format!("{}", r#type), control)
    }

    #[test]
    fn should_format_rust_function_type() {
        let r#type = Type::Function {
            arguments: vec![Type::Identifier {
                identifier: String::from("i64"),
            }],
            output: Box::new(Type::Identifier {
                identifier: String::from("i64"),
            }),
        };
        let control = String::from("fn(i64) -> i64");
        assert_eq!(format!("{}", r#type), control)
    }

    #[test]
    fn should_format_rust_closure_type() {
        let r#type = Type::Closure {
            arguments: vec![Type::Identifier {
                identifier: String::from("i64"),
            }],
            output: Box::new(Type::Identifier {
                identifier: String::from("()"),
            }),
        };
        let control = String::from("impl Fn(i64)");
        assert_eq!(format!("{}", r#type), control)
    }

    #[test]
    fn should_format_rust_generic_type() {
        let r#type = Type::Generic {
            generic: Box::new(Type::Identifier {
                identifier: String::from("Option"),
            }),
            arguments: vec![Type::Identifier {
                identifier: String::from("i64"),
            }],
        };
        let control = String::from("Option<i64>");
        assert_eq!(format!("{}", r#type), control)
    }

    #[test]
    fn should_format_rust_reference_type() {
        let r#type = Type::Reference {
            mutable: true,
            element: Box::new(Type::Identifier {
                identifier: String::from("i64"),
            }),
        };
        let control = String::from("&mut i64");
        assert_eq!(format!("{}", r#type), control)
    }

    #[test]
    fn should_format_rust_tuple_type() {
        let r#type = Type::Tuple {
            elements: vec![Type::Identifier {
                identifier: String::from("i64"),
            }],
        };
        let control = String::from("(i64)");
        assert_eq!(format!("{}", r#type), control)
    }
}
