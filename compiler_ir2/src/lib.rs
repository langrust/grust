#[macro_use]
pub mod prelude;

mod defs;

pub mod execution_machine;
pub mod ir1_into_ir2;
pub mod state_machine;

pub mod import {
    pub use crate::prelude::ir2::{self, Ir1IntoIr2};
}
