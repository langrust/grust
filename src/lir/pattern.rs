use crate::common::constant::Constant;

use super::r#type::Type;

/// Rust patterns used in match expression, closure, etc.
pub enum Pattern {
    /// An identifier pattern: `ref x`.
    Identifier {
        /// Reference: `true` is reference, `false` is owned.
        reference: bool,
        /// Mutability: `true` is mutable, `false` is immutable.
        mutable: bool,
        /// The identifier.
        identifier: String,
    },
    /// A literal pattern: `1` or `"hello world"`.
    Literal {
        /// The literal.
        literal: Constant,
    },
    /// A reference pattern: `&mut x`.
    Reference {
        /// Mutability: `true` is mutable, `false` is immutable.
        mutable: bool,
        /// The referenced pattern.
        pattern: Box<Pattern>,
    },
    /// A structure literal pattern: `Point { x, .. }`.
    Structure {
        /// The name of the structure.
        name: String,
        /// The filled fields.
        fields: Vec<FieldPattern>,
        /// The dots pattern.
        dots: bool,
    },
    /// A tuple pattern: `(x, y)`
    Tuple {
        /// Elements of the tuple.
        elements: Vec<Pattern>,
    },
    /// A typed pattern: `x: i64`.
    Typed {
        /// The typed pattern.
        pattern: Box<Pattern>,
        /// The type.
        r#type: Type,
    },
    /// The default pattern: `_`.
    Default,
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Identifier {
                reference,
                mutable,
                identifier,
            } => {
                let reference = if *reference { "ref " } else { "" };
                let mutable = if *mutable { "mut " } else { "" };
                write!(f, "{}{}{}", reference, mutable, identifier)
            }
            Pattern::Literal { literal } => write!(f, "{}", literal),
            Pattern::Reference { mutable, pattern } => {
                let mutable = if *mutable { "mut " } else { "" };
                write!(f, "&{}{}", mutable, pattern)
            }
            Pattern::Structure { name, fields, dots } => {
                let fields = fields
                    .iter()
                    .map(|field| format!("{field}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let dots = if *dots { ", .." } else { "" };
                write!(f, "{} {{ {}{} }}", name, fields, dots)
            }
            Pattern::Tuple { elements } => {
                let elements = elements
                    .iter()
                    .map(|element| format!("{element}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "({})", elements)
            }
            Pattern::Typed { pattern, r#type } => write!(f, "{}: {}", pattern, r#type),
            Pattern::Default => write!(f, "_"),
        }
    }
}

/// A structure's field filled with a pattern.
pub struct FieldPattern {
    /// Name of the field.
    pub name: String,
    /// Pattern stored in the field.
    pub pattern: Pattern,
}

impl std::fmt::Display for FieldPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.pattern.to_string() == self.name {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}: {}", self.name, self.pattern)
        }
    }
}

#[cfg(test)]
mod fmt {
    use crate::{
        common::{constant::Constant, r#type::Type as DSLType},
        lir::{pattern::FieldPattern, r#type::Type},
    };

    use super::Pattern;

    #[test]
    fn should_format_default_pattern() {
        let pattern = Pattern::Default;
        let control = String::from("_");
        assert_eq!(format!("{}", pattern), control)
    }

    #[test]
    fn should_format_identifier_pattern() {
        let pattern = Pattern::Identifier {
            reference: true,
            mutable: true,
            identifier: String::from("x"),
        };
        let control = String::from("ref mut x");
        assert_eq!(format!("{}", pattern), control)
    }

    #[test]
    fn should_format_literal_pattern() {
        let pattern = Pattern::Literal {
            literal: Constant::Integer(1),
        };
        let control = String::from("1i64");
        assert_eq!(format!("{}", pattern), control)
    }

    #[test]
    fn should_format_reference_pattern() {
        let pattern = Pattern::Reference {
            mutable: true,
            pattern: Box::new(Pattern::Identifier {
                reference: false,
                mutable: false,
                identifier: String::from("x"),
            }),
        };
        let control = String::from("&mut x");
        assert_eq!(format!("{}", pattern), control)
    }

    #[test]
    fn should_format_tuple_pattern() {
        let pattern = Pattern::Tuple {
            elements: vec![
                Pattern::Identifier {
                    reference: false,
                    mutable: false,
                    identifier: String::from("x"),
                },
                Pattern::Identifier {
                    reference: false,
                    mutable: false,
                    identifier: String::from("y"),
                },
            ],
        };
        let control = String::from("(x, y)");
        assert_eq!(format!("{}", pattern), control)
    }

    #[test]
    fn should_format_type_pattern() {
        let pattern = Pattern::Typed {
            pattern: Box::new(Pattern::Identifier {
                reference: false,
                mutable: false,
                identifier: String::from("x"),
            }),
            r#type: Type::Reference(DSLType::Integer),
        };
        let control = String::from("x: &i64");
        assert_eq!(format!("{}", pattern), control)
    }

    #[test]
    fn should_format_structure_pattern() {
        let pattern = Pattern::Structure {
            name: String::from("Point"),
            fields: vec![
                FieldPattern {
                    name: String::from("x"),
                    pattern: Pattern::Identifier {
                        reference: false,
                        mutable: false,
                        identifier: String::from("x"),
                    },
                },
                FieldPattern {
                    name: String::from("y"),
                    pattern: Pattern::Literal {
                        literal: Constant::Integer(1),
                    },
                },
            ],
            dots: true,
        };
        let control = String::from("Point { x, y: 1i64, .. }");
        assert_eq!(format!("{}", pattern), control)
    }
}
