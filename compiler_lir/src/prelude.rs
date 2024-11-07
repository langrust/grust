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

pub use compiler_ast::import::*;
pub use compiler_common::import::*;
pub use compiler_hir::import::*;

pub mod lir {
    pub use crate::{
        block::Block,
        contract::{self, Contract},
        execution_machine::{self, ExecutionMachine},
        expression::*,
        hir_into_lir::HirIntoLir,
        item::{self, Enumeration, Function, Import, Item, Structure},
        pattern::Pattern,
        project::Project,
        state_machine::{self, StateMachine},
        stmt::Stmt,
    };
}

pub use lir::*;
