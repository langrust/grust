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
macro_rules! bail {
    { $e:expr } => { return Err($e.into()) };
    { $($fmt:tt)* } => { return Err(format!($($fmt)*).into()) };
}

pub use crate::bail;

pub use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

pub mod syn {
    pub use syn::{
        braced, bracketed, custom_keyword, parenthesized,
        parse::{Parse, ParseStream},
        parse_macro_input, parse_quote, parse_str,
        punctuated::{self, Punctuated},
        spanned::Spanned,
        Token, *,
    };

    /// Alias for `syn`'s notion of result.
    pub type Res<T> = syn::Result<T>;
}

pub use syn::{
    braced, bracketed, custom_keyword, parenthesized, parse_macro_input, parse_quote, Ident,
    ParseStream, Token,
};

pub use crate::{
    codespan_reporting, conf, constant::Constant, convert_case::to_camel_case, error::*, graph,
    hash_map::*, itertools, keyword, lazy_static::lazy_static, location::Location, macro2, mk_new,
    once_cell, operator, petgraph, quote, rustc_hash, safe_index, scope::Scope, serde, strum,
    synced, typ::Typ,
};

/// Provides context-dependent a *less than* (`lt`) relation.
pub trait Lt {
    /// Type of the context.
    type Ctx;

    /// Compares two `Self`-values.
    fn lt(&self, other: &Self, ctx: &Self::Ctx) -> bool;
}

/// Extension over `Iterator`s.
pub trait IteratorExt: Sized {
    /// Type of the items the iterator is for.
    type I;

    /// Pairwise fold.
    fn pairwise<Acc>(self, init: Acc, f: impl FnMut(Acc, Self::I, Self::I) -> Acc) -> Acc;

    /// Checks that an iterator is sorted.
    ///
    /// Weird name because of a warning that `is_sorted` may be used by the compiler one day.
    fn check_sorted(self) -> bool
    where
        Self::I: Ord,
    {
        self.pairwise(true, |okay, lft, rgt| okay && lft <= rgt)
    }
}
impl<T> IteratorExt for T
where
    T: Iterator,
    T::Item: Copy,
{
    type I = T::Item;
    fn pairwise<Acc>(self, init: Acc, mut f: impl FnMut(Acc, T::Item, T::Item) -> Acc) -> Acc {
        self.fold((None, init), |(prev, acc), curr| {
            if let Some(prev) = prev {
                (Some(curr), f(acc, prev, curr))
            } else {
                (Some(curr), acc)
            }
        })
        .1
    }
}

/// Extension over `Vec`tors.
pub trait VecExt<T> {
    /// If self has length `1`, extracts the only value and returns it.
    fn pop_single(&mut self) -> Option<T>;
}

impl<T> VecExt<T> for Vec<T> {
    fn pop_single(&mut self) -> Option<T> {
        if self.len() == 1 {
            let res = self
                .pop()
                .expect("popping a vector of length 1 is always legal");
            debug_assert_eq!(self.len(), 0);
            Some(res)
        } else {
            None
        }
    }
}
