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

pub use compiler_ast::import::*;

pub mod hir {
    pub use crate::{
        ast_into_hir::IntoHir,
        ast_store::{
            AstStore, AstStoreEventPattern, AstStorePattern, AstStoreSignals, AstStoreStmtPattern,
        },
        ast_typing::Typing,
        defs::{
            component::{Component, ComponentDefinition, ComponentImport},
            contract::{self, Contract},
            ctx,
            dependencies::Dependencies,
            expr::{self, Expr},
            file::File,
            flow, from_ast,
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
    };
}

pub use hir::*;
