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
                write!(f, "{} {{{}{}}}", name, fields, dots)
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
        write!(f, "{}: {}", self.name, self.pattern)
    }
}
