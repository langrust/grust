use crate::lir::{expression::Expression, pattern::Pattern};

/// A statement declaration.
#[derive(Debug, PartialEq)]
pub enum Statement {
    /// A let-statement creating one variable: `let x = y + 1;`.
    Let {
        /// The variables created.
        pattern: Pattern,
        /// The expression associated to the variable.
        expression: Expression,
    },
    /// A returned expression.
    ExpressionLast {
        /// The returned expression.
        expression: Expression,
    },
}
