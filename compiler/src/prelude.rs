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
    ext::*,
    frontend::{self, hir_from_ast::HIRFromAST},
    hir,
};

/// Translation to Rust AST with additional information.
pub trait ToRustAstWith<Data>: Sized {
    type Output;
    fn to_rust_with(self, data: Data) -> Self::Output;
}

/// Auto-implemented for `T: ToRustAstWith<()>`.
pub trait ToRustAst: ToRustAstWith<()> + Sized {
    fn to_rust(self) -> Self::Output;
}

impl<T> ToRustAst for T
where
    T: ToRustAstWith<()>,
{
    fn to_rust(self) -> Self::Output {
        self.to_rust_with(())
    }
}
