prelude! { Expr, Pattern }

/// A statement declaration.
#[derive(Debug, PartialEq)]
pub enum Stmt {
    /// A let-statement creating one variable: `let x = y + 1;`.
    Let {
        /// The variables created.
        pattern: Pattern,
        /// The expression associated to the variable.
        expression: Expr,
    },
    /// A returned expression.
    ExprLast {
        /// The returned expression.
        expression: Expr,
    },
}

mk_new! { impl Stmt =>
    Let: let_binding {
        pattern: Pattern,
        expression: Expr,
    }
    ExprLast: expression_last { expression: Expr }
}
