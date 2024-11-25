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
        defs::{
            block::Block,
            contract::{self, Contract},
            expression::*,
            item::{self, Enumeration, Function, Import, Item, Structure},
            pattern::Pattern,
            project::Project,
            stmt::Stmt,
        },
        execution_machine::{self, ExecutionMachine},
        ir1_into_ir2::{self, Ir1IntoIr2, TriggersGraph},
        state_machine::{self, StateMachine},
    };
}

pub use ir2::*;
