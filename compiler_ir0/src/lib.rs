pub extern crate compiler_common as common;

#[macro_use]
pub mod prelude;

mod defs;
mod parsing;
pub mod symbol;

pub mod import {
    pub use crate::prelude::ir0;
    pub use ir0::{Ast, SymbolTable};
}
