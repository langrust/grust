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

// #[macro_export]
// macro_rules! bail {
//     { $e:expr } => { return Err($e.into()) };
//     { $($fmt:tt)* } => { return Err(format!($($fmt)*).into()) };
// }

// pub use crate::bail;

pub use std::{
    collections::{BTreeMap, BTreeSet},
    error,
    fmt::Display,
    hash::Hash,
    ops,
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

pub use either::{Either, IntoEither};
pub use proc_macro2::{Span, TokenStream as TokenStream2};
pub use quote::{format_ident, quote_spanned, ToTokens};
pub use serde::{Deserialize, Serialize};
pub use syn::{
    braced, bracketed, custom_keyword, parenthesized, parse_macro_input, parse_quote, Ident,
    ParseStream, Token,
};

pub use crate::{
    bad,
    // error::*,
    bail,
    check,
    codespan_reporting,
    conf,
    constant::Constant,
    convert_case::to_camel_case,
    err::*,
    error,
    graph,
    hash_map::*,
    itertools,
    keyword,
    lazy_static::lazy_static,
    lerror,
    lnote,
    macro2,
    mk_new,
    note,
    once_cell,
    op::{BOp, OtherOp, UOp},
    petgraph,
    quote,
    rustc_hash,
    safe_index,
    scope::Scope,
    serde,
    strum,
    synced,
    typ::Typ,
};

#[derive(Debug, Clone, Copy)]
pub struct Loc {
    pub span: Span,
}
impl Loc {
    pub fn test_dummy() -> Self {
        Self {
            span: Span::mixed_site(),
        }
    }
    pub fn call_site() -> Self {
        Self {
            span: Span::call_site(),
        }
    }
    pub fn mixed_site() -> Self {
        Self {
            span: Span::mixed_site(),
        }
    }
    pub fn try_join(self, that: impl Into<Self>) -> Option<Self> {
        self.span.join(that.into().span).map(Loc::from)
    }
    pub fn join(self, that: impl Into<Self>) -> Self {
        self.try_join(that).unwrap_or(self)
    }
}
impl PartialEq for Loc {
    fn eq(&self, other: &Self) -> bool {
        // #TODO: that's pretty bad, but we can't have `PartialEq` on `Span` itself...
        format!("{:?}", self.span) == format!("{:?}", other.span)
    }
}
impl Eq for Loc {}
impl std::hash::Hash for Loc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // #TODO: that's pretty bad, but we can't have `Hash` on `Span` itself...
        format!("{:?}", self.span).hash(state)
    }
}
impl std::ops::Deref for Loc {
    type Target = Span;
    fn deref(&self) -> &Self::Target {
        &self.span
    }
}
impl From<Span> for Loc {
    fn from(span: Span) -> Self {
        Self { span }
    }
}
impl Into<Span> for Loc {
    fn into(self) -> Span {
        self.span
    }
}

pub fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

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
        self.fold((None, init), |(prev, acc), cur| {
            if let Some(prev) = prev {
                (Some(cur), f(acc, prev, cur))
            } else {
                (Some(cur), acc)
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
