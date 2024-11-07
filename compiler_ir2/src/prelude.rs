//! Basic types, helpers and re-exports.

/// Imports the compiler prelude.
#[macro_export]
macro_rules! prelude {
    { just $($imports:tt)* } => {
        use $crate::prelude::{$($imports)*};
    };
    { $($imports:tt)* } => {
        use $crate::prelude::{*, $($imports)*};
    };
}

pub use compiler_common::import::*;
pub use compiler_ir0::import::*;
pub use compiler_ir1::import::*;

pub mod ir2 {
    pub use crate::{
        block::Block,
        contract::{self, Contract},
        execution_machine::{self, ExecutionMachine},
        expression::*,
        ir1_into_ir2::Ir1IntoIr2,
        item::{self, Enumeration, Function, Import, Item, Structure},
        pattern::Pattern,
        project::Project,
        state_machine::{self, StateMachine},
        stmt::Stmt,
    };
}

pub use ir2::*;
