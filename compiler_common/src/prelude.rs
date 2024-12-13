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
macro_rules! todoo {
    {  } => {
        unimplemented!("at `{}:{}`", file!(), line!())
    };
    { $($blah:tt)* } => {
        unimplemented!("at `{}:{}`, {}", file!(), line!(), format!($($blah)*))
    };
}

#[macro_export]
macro_rules! noteln {
    { $($stuff:tt)* } => {
        error!($($stuff)*).emit_note()
    }
}

#[macro_export]
macro_rules! res_vec {
    { $len:expr, $e:expr $(,)? } => {{
        let mut vec = Vec::with_capacity($len);
        for res in $e {
            vec.push(res?);
        }
        vec
    }};
    { $e:expr $(,)? } => {{
        for res in $e {
            res?;
        }
    }};
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
    time::{Duration, Instant},
};

pub mod syn {
    pub use syn::{
        braced, bracketed, custom_keyword, parenthesized,
        parse::{Parse, ParseStream},
        parse_macro_input, parse_quote, parse_quote_spanned, parse_str,
        punctuated::{self, Punctuated},
        spanned::Spanned,
        Token, *,
    };

    /// Alias for `syn`'s notion of result.
    pub type Res<T> = syn::Result<T>;
}

pub use either::{Either, IntoEither};
pub use macro1;
pub use macro2::{Span, TokenStream as TokenStream2};
pub use quote::{format_ident, quote_spanned, ToTokens};
pub use serde::{Deserialize, Serialize};
pub use syn::{
    braced, bracketed, custom_keyword, parenthesized, parse_macro_input, parse_quote,
    parse_quote_spanned, Ident, ParseStream, Token,
};

pub use crate::{
    bad,
    // error::*,
    bail,
    check,
    // codespan_reporting,
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
    noteln,
    once_cell,
    op::{BOp, OtherOp, UOp},
    petgraph,
    quote,
    res_vec,
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
    pub fn test_id(id: impl AsRef<str>) -> Ident {
        Ident::new(id.as_ref(), Self::test_dummy().into())
    }
    pub fn test_dummy() -> Self {
        Self {
            span: Span::mixed_site(),
        }
    }
    pub fn nu_call_site() -> Self {
        Self {
            span: Span::call_site(),
        }
    }
    pub fn builtin() -> Self {
        Self {
            span: Span::call_site(),
        }
    }
    pub fn builtin_id(id: impl AsRef<str>) -> Ident {
        Ident::new(id.as_ref(), Self::builtin().into())
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
pub trait IteratorExt: IntoIterator + Sized {
    /// Pairwise fold.
    fn pairwise<Acc>(self, init: Acc, f: impl FnMut(Acc, Self::Item, Self::Item) -> Acc) -> Acc
    where
        Self::Item: Clone;

    /// Checks that an iterator is sorted.
    ///
    /// Weird name because of a warning that `is_sorted` may be used by the compiler one day.
    fn check_sorted(self) -> bool
    where
        Self::Item: Ord + Clone,
    {
        self.pairwise(true, |okay, lft, rgt| okay && lft <= rgt)
    }

    fn collect_vec(self) -> Vec<Self::Item> {
        self.into_iter().collect()
    }
}
impl<T> IteratorExt for T
where
    T: IntoIterator,
{
    fn pairwise<Acc>(self, init: Acc, mut f: impl FnMut(Acc, T::Item, T::Item) -> Acc) -> Acc
    where
        Self::Item: Clone,
    {
        self.into_iter()
            .fold((None, init), |(prev, acc), cur| {
                if let Some(prev) = prev {
                    (Some(cur.clone()), f(acc, prev, cur))
                } else {
                    (Some(cur), acc)
                }
            })
            .1
    }
}
pub trait ResIteratorExt<E>: IntoIterator<Item = Result<(), E>> {
    fn collect_res(self) -> Result<(), E>;
}
impl<T, E> ResIteratorExt<E> for T
where
    T: IntoIterator<Item = Result<(), E>>,
{
    fn collect_res(self) -> Result<(), E> {
        self.into_iter().collect()
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

pub trait HasLoc {
    fn loc(&self) -> Loc;
}
impl HasLoc for Ident {
    fn loc(&self) -> Loc {
        self.span().into()
    }
}

pub trait MaybeHasLoc {
    fn loc_opt(&self) -> Option<Loc>;
}

/// Byte-level levenshtein distance, not sure this makes sense for unicode.
///
/// Stackless version of <https://en.wikipedia.org/wiki/Levenshtein_distance#Recursive>.
pub fn levenshtein(s1: impl AsRef<str>, s2: impl AsRef<str>) -> usize {
    struct TwoBytes<'a>(&'a [u8], &'a [u8]);
    enum Frame<'a> {
        TwoLeft {
            next: TwoBytes<'a>,
            last: TwoBytes<'a>,
        },
        OneLeft {
            min: usize,
            next: TwoBytes<'a>,
        },
        NoneLeft {
            min: usize,
        },
    }
    let (mut s1, mut s2) = (s1.as_ref().as_bytes(), s2.as_ref().as_bytes());
    let mut stack: Vec<Frame> = Vec::with_capacity(s1.len().max(s2.len()));
    'measure: loop {
        // println!("measure");
        // println!("- `{}`", String::from_utf8_lossy(s1));
        // println!("- `{}`", String::from_utf8_lossy(s2));
        let mut distance = if s1.is_empty() {
            s2.len()
        } else if s2.is_empty() {
            s1.len()
        } else if s1[0] == s2[0] {
            s1 = &s1[1..];
            s2 = &s2[1..];
            continue 'measure;
        } else {
            stack.push(Frame::TwoLeft {
                next: TwoBytes(&s1[1..], s2),
                last: TwoBytes(&s1[1..], &s2[1..]),
            });
            s2 = &s2[1..];
            continue 'measure;
        };

        'unstack: loop {
            let TwoBytes(next1, next2) = match stack.pop() {
                None => return distance,
                Some(Frame::TwoLeft { next, last }) => {
                    stack.push(Frame::OneLeft {
                        min: distance,
                        next: last,
                    });
                    next
                }
                Some(Frame::OneLeft { min, next }) => {
                    stack.push(Frame::NoneLeft {
                        min: distance.min(min),
                    });
                    next
                }
                Some(Frame::NoneLeft { min }) => {
                    distance = 1 + distance.min(min);
                    continue 'unstack;
                }
            };
            s1 = next1;
            s2 = next2;
            continue 'measure;
        }
    }
}

#[test]
fn test_levenshtein() {
    fn check(s1: &str, s2: &str, exp: usize) {
        let l = levenshtein(s1, s2);
        println!();
        println!("- {} =?= {}", l, exp);
        println!("  `{}`", s1);
        println!("  `{}`", s2);
        assert_eq!(l, exp);
        println!("  ✅")
    }
    check("a", "a", 0);
    check("a", "b", 1);
    check("i12", "i76", 2);
    check("kitten", "sitting", 3);
    check("uninformed", "uniformed", 1);
}

pub struct Stats {
    vec: Vec<(String, Duration)>,
}
impl Stats {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vec: Vec::with_capacity(capacity),
        }
    }
    pub fn new() -> Self {
        Self::with_capacity(10)
    }

    pub fn timed<T>(&mut self, desc: impl Into<String>, run: impl FnOnce() -> T) -> T {
        let start = Instant::now();
        let res = run();
        self.vec.push((desc.into(), Instant::now() - start));
        res
    }

    pub fn pretty(&self) -> String {
        let max_key_len = if let Some(max) = self.vec.iter().map(|(s, _)| s.chars().count()).max() {
            max
        } else {
            return String::new();
        };
        let mut string = String::with_capacity(200);
        let mut sep = "| ";
        for (s, d) in self.vec.iter() {
            string.push_str(sep);
            for _ in s.chars().count()..max_key_len {
                string.push(' ');
            }
            string.push_str(s);
            string.push_str(" | ");
            let secs = format!("{}.{:0>3}", d.as_secs(), d.subsec_nanos());
            for _ in secs.len()..15 {
                string.push(' ');
            }
            string.extend([secs].into_iter());
            string.push_str(" |");
            sep = "\n| ";
        }
        string
    }
}
