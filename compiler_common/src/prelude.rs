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

pub use crate::{
    codespan_reporting, conf, constant::Constant, convert_case::to_camel_case, error::*, graph,
    hash_map::*, itertools, keyword, lazy_static::lazy_static, location::Location, macro2, mk_new,
    once_cell, operator, petgraph, quote, r#type::Typ, rustc_hash, scope::Scope, serde, strum, syn,
};
