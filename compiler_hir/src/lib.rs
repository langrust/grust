#[macro_use]
pub mod prelude;

mod defs;

pub use defs::*;

mod ast_into_hir;
mod ast_store;
mod ast_typing;
mod dependency_graph;

pub mod import {
    pub use crate::prelude::hir::{self, AstStore, IntoHir, Typing};
}
