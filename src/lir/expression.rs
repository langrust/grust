use crate::common::constant::Constant;

/// Rust expressions.
pub enum Expression {
    /// A literal expression: `1` or `"hello world"`.
    Literal {
        /// The literal.
        literal: Constant,
    },
    /// An identifier call: `x`.
    Identifier {
        /// The identifier.
        identifier: String,
    },
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Literal { literal } => write!(f, "{literal}"),
            Expression::Identifier { identifier } => write!(f, "{identifier}"),
        }
    }
}
