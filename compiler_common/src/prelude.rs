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
    codespan_reporting, conf, constant::Constant, convert_case::to_camel_case, equiv, error::*,
    graph, hash_map::*, itertools, keyword, lazy_static::lazy_static, location::Location, macro2,
    mk_new, once_cell, operator, petgraph, quote, r#type::Typ, rustc_hash, safe_index,
    scope::Scope, serde, strum, syn,
};

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
