use crate::lir::expression::Expression;

/// A `let` binding: `let x: u64 = 5`.
pub struct Let {
    /// Reference: `true` is reference, `false` is owned.
    pub reference: bool,
    /// Mutability: `true` is mutable, `false` is immutable.
    pub mutable: bool,
    /// The created variable.
    pub identifiant: String,
    /// The associated expression.
    pub expression: Expression,
}

impl std::fmt::Display for Let {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reference = if self.reference { "ref " } else { "" };
        let mutable = if self.mutable { "mut " } else { "" };
        write!(
            f,
            "let {}{}{} = {};",
            reference, mutable, self.identifiant, self.expression
        )
    }
}

#[cfg(test)]
mod fmt {
    use crate::{common::constant::Constant, lir::statement::r#let::Let};

    use super::Expression;

    #[test]
    fn should_format_let_binding() {
        let let_binding = Let {
            reference: false,
            mutable: true,
            identifiant: String::from("x"),
            expression: Expression::Literal {
                literal: Constant::Integer(1),
            },
        };
        let control = String::from("let mut x = 1i64;");
        assert_eq!(format!("{}", let_binding), control)
    }
}
