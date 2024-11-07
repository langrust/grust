pub extern crate compiler_ast as ast;

pub use ast::common;

#[macro_use]
pub mod prelude;

mod block;
mod expression;
mod hir_into_lir;
mod pattern;
mod stmt;

pub mod contract;
pub mod execution_machine;
pub mod item;
pub mod project;
pub mod state_machine;

pub mod import {
    pub use crate::prelude::lir::{self, HirIntoLir};
}
