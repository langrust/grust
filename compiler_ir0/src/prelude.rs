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

pub mod ir0 {
    pub use crate::{
        defs::{
            contract::{self, Contract},
            equation::{self, Eq, ReactEq},
            expr::{self, Expr},
            interface::{self, ExtFunDecl, ExtCompDecl, FlowExport, FlowImport, Service, TimeRange},
            stmt::{self, Stmt},
            stream, Ast, Colon, Component, Ctx, Function, Item, Top, Typedef, ConstDecl,
        },
        symbol,
    };
}

pub use crate::parsing::ParsePrec;
pub use ir0::*;
