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

/// Iterates over a collection with a special action if there is only one element.
///
/// # Examples
///
/// ```rust
/// # use compiler_common::iter_1;
/// fn run(vec: Vec<usize>) -> usize {
///     iter_1!(
///         // `Iterator + ExactSizeIterator` to work on
///         vec.into_iter(),
///         // action when the length isn't one
///         |coll| coll.sum(),
///         // action when the iterator contains a single element
///         |elem| elem * 10
///     )
/// }
/// let coll_1 = vec![2];
/// assert_eq!( run(coll_1), 20 );
/// let coll_n = vec![2, 3, 4];
/// assert_eq!( run(coll_n), 9 );
/// ```
#[macro_export]
macro_rules! iter_1 {
    { $e:expr, |$iter:ident| $iter_do:expr, |$elem:ident| $one_do:expr $(,)? } => {{
        let mut iter = $e;
        if iter.len() == 1 {
            let $elem = iter.next().expect("len is `1`");
            $one_do
        } else {
            let $iter = iter;
            $iter_do
        }
    }}
}

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
    iter_1,
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
    vec: Vec<(String, Duration, Option<Stats>)>,
}
pub struct StatsItem {
    start: Instant,
    desc: String,
}
pub struct StatsMut<'a> {
    inner: &'a mut Stats,
}
impl<'a> std::ops::Deref for StatsMut<'a> {
    type Target = Stats;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}
impl<'a> std::ops::DerefMut for StatsMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
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

    pub fn as_mut(&mut self) -> StatsMut {
        StatsMut { inner: self }
    }

    pub fn start(&self, desc: impl Into<String>) -> StatsItem {
        StatsItem {
            start: Instant::now(),
            desc: desc.into(),
        }
    }

    pub fn end(&mut self, i: StatsItem) {
        self.vec.push((i.desc, Instant::now() - i.start, None))
    }
    pub fn augment_end(&mut self, i: StatsItem) {
        self.augment(i.desc, Instant::now() - i.start, None)
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn indent(&mut self) {
        for (desc, _, sub_opt) in self.vec.iter_mut() {
            *desc = format!("  {}", desc);
            if let Some(sub) = sub_opt {
                sub.indent()
            }
        }
    }

    pub fn augment(&mut self, desc: impl Into<String>, time: Duration, sub_opt: Option<Stats>) {
        let desc = desc.into();
        for (desc2, time2, sub_opt2) in &mut self.vec {
            if &desc == desc2 {
                *time2 = time + *time2;
                match (sub_opt, sub_opt2.as_mut()) {
                    (None, _) => (),
                    (Some(sub), None) => *sub_opt2 = Some(sub),
                    (Some(sub), Some(sub2)) => sub2.augment_merge(sub),
                }
                return ();
            }
        }
        self.vec.push((desc, time, sub_opt));
    }

    pub fn augment_merge(&mut self, that: Self) {
        for (d, t, s) in that.vec {
            self.augment(d, t, s)
        }
    }

    pub fn augment_timed_with<T>(
        &mut self,
        desc: impl Into<String>,
        run: impl FnOnce(StatsMut) -> T,
    ) -> T {
        let mut sub = Self::new();
        let start = Instant::now();
        let res = run(sub.as_mut());
        let time = Instant::now() - start;
        let sub_opt = if sub.is_empty() {
            None
        } else {
            sub.indent();
            Some(sub)
        };
        self.augment(desc, time, sub_opt);
        res
    }

    pub fn timed_with<T>(&mut self, desc: impl Into<String>, run: impl FnOnce(StatsMut) -> T) -> T {
        let start = Instant::now();
        let mut sub = Self::new();
        let res = run(sub.as_mut());
        let sub_opt = if sub.is_empty() {
            None
        } else {
            sub.indent();
            Some(sub)
        };
        self.vec
            .push((desc.into(), Instant::now() - start, sub_opt));
        res
    }

    pub fn timed<T>(&mut self, desc: impl Into<String>, run: impl FnOnce() -> T) -> T {
        self.timed_with(desc, |_| run())
    }

    pub fn max_key_len(&self) -> usize {
        let mut max = 0;
        for (s, _, sub) in &self.vec {
            max = max.max(s.chars().count());
            if let Some(sub) = sub {
                max = max.max(sub.max_key_len());
            }
        }
        max
    }

    pub fn pretty(&self) -> Option<String> {
        let max_depth = conf::stats_depth();
        if max_depth == 0 {
            None
        } else {
            Some(self.pretty_aux(self.max_key_len(), 1, max_depth))
        }
    }

    fn pretty_aux(&self, max_key_len: usize, depth: usize, max_depth: usize) -> String {
        let mut string = String::with_capacity(200);
        let mut sep = "| ";
        for (desc, duration, sub_opt) in self.vec.iter() {
            string.push_str(sep);
            string.push_str(desc);
            for _ in desc.chars().count()..max_key_len {
                string.push(' ');
            }
            string.push_str(" | ");
            let secs = format!("{}.{:0>9}", duration.as_secs(), duration.subsec_nanos());
            for _ in secs.len()..15 {
                string.push(' ');
            }
            string.extend([secs].into_iter());
            string.push_str(" |");
            if let Some(sub) = sub_opt {
                let depth = depth + 1;
                if depth <= max_depth {
                    string.push('\n');
                    string.push_str(&sub.pretty_aux(max_key_len, depth, max_depth));
                }
            }
            sep = "\n| ";
        }
        string
    }
}
