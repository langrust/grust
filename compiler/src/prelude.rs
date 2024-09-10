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

pub use ast::Ast;
pub use compiler_lir::{
    ast::prelude as ast,
    common::prelude::{mk_new, *},
    prelude as lir,
    prelude::SymbolTable,
};

pub use crate::{
    backend,
    ext::*,
    frontend::{self, hir_from_ast::HIRFromAST},
    hir,
};
