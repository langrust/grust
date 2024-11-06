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

pub use compiler_common::prelude::{mk_new, *};

pub use crate::{
    component::{Component, ComponentImport},
    contract::{self, Contract},
    equation::{self, Eq, ReactEq},
    expr::{self, Expr},
    function::Function,
    interface::{self, Constrains, FlowExport, FlowImport, Service},
    stmt::{self, Stmt},
    stream,
    symbol::{self, SymbolTable},
    typedef::Typedef,
    Ast, Colon, Item,
};
