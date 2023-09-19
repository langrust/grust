use crate::lir::{expression::Expression, pattern::Pattern};

/// A `let` binding: `let x: u64 = 5`.
pub struct Let {
    /// The created pattern variables.
    pub pattern: Pattern,
    /// The associated expression.
    pub expression: Expression,
}

impl std::fmt::Display for Let {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {} = {};", self.pattern, self.expression)
    }
}

#[cfg(test)]
mod fmt {
    use crate::{
        common::constant::Constant,
        lir::{pattern::Pattern, statement::r#let::Let},
    };

    use super::Expression;

    #[test]
    fn should_format_let_binding() {
        let let_binding = Let {
            pattern: Pattern::Identifier {
                reference: false,
                mutable: true,
                identifier: String::from("x"),
            },
            expression: Expression::Literal {
                literal: Constant::Integer(1),
            },
        };
        let control = String::from("let mut x = 1i64;");
        assert_eq!(format!("{}", let_binding), control)
    }
}
