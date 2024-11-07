#[macro_use]
pub mod prelude;

mod defs;

pub use defs::*;

mod dependency_graph;
mod ir0_into_ir1;
mod ir0_store;
mod typing;

pub mod import {
    pub use crate::prelude::ir1::{self, Ir0IntoIr1, Ir0Store, Typing};
}
