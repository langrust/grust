use crate::lir::expression::Expression;

/// A statement declaration.
#[derive(Debug, PartialEq)]
pub enum Statement {
    /// A let-statement creating one variable: `let x = y + 1;`.
    Let {
        /// The variable created.
        identifier: String,
        /// The expression associated to the variable.
        expression: Expression,
    },
    /// A returned expression.
    ExpressionLast {
        /// The returned expression.
        expression: Expression,
    },
}
