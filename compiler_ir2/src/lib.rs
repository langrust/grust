#[macro_use]
pub mod prelude;

mod block;
mod expression;
mod ir1_into_ir2;
mod pattern;
mod stmt;

pub mod contract;
pub mod execution_machine;
pub mod item;
pub mod project;
pub mod state_machine;

pub mod import {
    pub use crate::prelude::ir2::{self, Ir1IntoIr2};
}
