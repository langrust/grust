use crate::mir::expression::Expression;

/// A statement declaration.
pub enum Statement {
    /// A let-statement creating one variable: `let x = y + 1;`.
    Let {
        /// The variable created.
        identifier: String,
        /// The expression associated to the variable.
        expression: Expression,
    },
    /// A let-statement creating a tuple of variables: `let (o, new_state) = self.state.step(inputs);`.
    LetTuple {
        /// The variables created.
        identifiers: Vec<String>,
        /// The expression associated to the variables.
        expression: Expression,
    },
    /// A returned expression.
    ExpressionLast {
        /// The returned expression.
        expression: Expression,
    },
}
