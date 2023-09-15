use crate::lir::{expression::Expression, item::Item, statement::r#let::Let};

/// LIR [Let](crate::lir::statement::r#let::Let) module.
pub mod r#let;

/// Rust statement.
pub enum Statement {
    /// A `let` binding.
    Let(Let),
    /// An item definition.
    Item(Item),
    /// An internal expression, endding with a semicolon.
    ExpressionIntern(Expression),
    /// The last expression, no semicolon.
    ExpressionLast(Expression),
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Let(r#let) => write!(f, "{}", r#let),
            Statement::Item(item) => write!(f, "{item}"),
            Statement::ExpressionIntern(expression) => write!(f, "{expression};"),
            Statement::ExpressionLast(expression) => write!(f, "{expression}"),
        }
    }
}
