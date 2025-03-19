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

#[macro_export]
macro_rules! tupleify {
    { $es:expr, $count:expr => $len_n:expr => $len_1:expr } => {
        if $count == 1 {
            $len_1
        } else {
            $len_n
        }
    };
    { $es:expr => $len_n:expr => $len_1:expr } => {
        $crate::tupleify! {
            $es, $es.len() => $len_n => $len_1
        }
    };
    { $es:expr $(, $count:expr)? } => {
        $crate::tupleify! {
            $es $(, $count)?
            => {
                let es = $es;
               $crate::prelude::parse_quote! { (#({#es}),*) }
            }
            => {
                let es = $es;
                $crate::prelude::parse_quote! { #({#es})* }
            }
        }
    };
    { $es:expr $(, $count:expr)? => $len_n:expr } => {
        $crate::tupleify! {
            $es $(, $count)?
            => $len_n
            => {
                let es = $es;
                $crate::prelude::parse_quote! { #({#es})* }
            }
        }
    };
    { $es:expr $(, $count:expr)? => => len_1:expr } => {
        $crate::tupleify! {
            $es $(, $count)?
            => {
                let es = #es;
                parse_quote! { (#({#es}),*) }
            }
            => $len_1
        }
    }
}

pub use compiler_common::import::{noErrorDesc, *};
pub use compiler_ir0::import::*;
pub use compiler_ir1::import::*;

pub mod ir2 {
    pub use crate::{
        defs::{
            block::Block,
            contract::{self, Contract},
            expression::*,
            item::{self, Enumeration, Function, Import, Item, Structure},
            para,
            pattern::Pattern,
            project::Project,
            stmt::Stmt,
        },
        execution_machine::{self, ExecutionMachine},
        ir1_into_ir2::{self, Ir1IntoIr2, TriggersGraph},
        state_machine::{self, StateMachine},
        tupleify,
    };
}

pub use ir2::*;
