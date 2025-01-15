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

pub mod ir1 {
    pub use crate::{
        defs::{
            component::{Component, ComponentDefinition, ComponentImport},
            contract::{self, Contract},
            ctx,
            dependencies::Dependencies,
            expr::{self, Expr},
            file::File,
            flow, from_ast, from_ast_timed,
            function::Function,
            identifier_creator::IdentifierCreator,
            interface::{self, Interface, Service},
            memory::{self, Memory},
            once_cell::OnceCell,
            pattern::{self, Pattern},
            stmt::{self, Stmt},
            stream,
            typedef::{self, Typedef},
        },
        dependencies::DepCtx,
        ir0_into_ir1::Ir0IntoIr1,
        ir0_store::{
            Ir0Store, Ir0StoreEventPattern, Ir0StorePattern, Ir0StoreSignals, Ir0StoreStmtPattern,
        },
        typing::Typing,
    };
}

pub use ir1::*;
