use crate::lir::{expression::Expression, item::Item, statement::r#let::Let};

/// LIR [Let](crate::lir::statement::r#let::Let) module.
pub mod r#let;

/// Rust statement.
pub enum Statement {
    /// A `let` binding.
    Let(Let),
    /// An item definition.
    Item(Item),
    /// An expression.
    Expression(Expression),
}
