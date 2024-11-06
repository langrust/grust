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

pub use compiler_ast::common::prelude::{mk_new, *};

pub use compiler_ast::prelude as ast;

pub use ast::{Ast, SymbolTable};

pub use crate::{
    block::Block,
    contract::{self, Contract},
    execution_machine::{self, ExecutionMachine},
    expression::*,
    item::{self, Enumeration, Function, Import, Item, Structure},
    pattern::Pattern,
    project::Project,
    state_machine::{self, StateMachine},
    stmt::Stmt,
};
