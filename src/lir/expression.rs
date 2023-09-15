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
    /// A structure literal expression: `Point { x: 1, y: 1 }`.
    Structure {
        /// The name of the structure.
        name: String,
        /// The filled fields.
        fields: Vec<FieldExpression>,
    },
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Literal { literal } => write!(f, "{literal}"),
            Expression::Identifier { identifier } => write!(f, "{identifier}"),
            Expression::Structure { name, fields } => {
                let fields = fields
                    .iter()
                    .map(|field| format!("{field}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{} {{{}}}", name, fields)
            }
        }
    }
}

/// A structure's field filled with an expression.
pub struct FieldExpression {
    /// Name of the field.
    pub name: String,
    /// Expression stored in the field.
    pub expression: Expression,
}

impl std::fmt::Display for FieldExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.expression)
    }
}
