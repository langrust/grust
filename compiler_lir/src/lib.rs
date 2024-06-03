pub extern crate compiler_ast as ast;

pub use ast::common;

#[macro_use]
pub mod prelude;

mod block;
mod stmt;

/// LIR [Contract](crate::lir::contract::Contract) module.
pub mod contract;

/// LIR [Expression](crate::lir::expression::Expression) module.
mod expression;

/// LIR [Item](crate::lir::item::Item) module.
pub mod item;

/// LIR [Pattern](crate::lir::pattern::Pattern) module.
pub mod pattern;

/// LIR [Project](crate::lir::project::Project) module.
pub mod project;
