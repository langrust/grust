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
pub use compiler_ir2::import::*;

pub use crate::TokenStream;

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
