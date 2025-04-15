//! Parallelization stuff.
//!
//! The [generic] sub-module contains a generic library to generate parallelization graphs from
//! statement-dependencies graphs.
//!
//! This module deals with the notion of [Weight] and provides values basic [weight] quantities,
//! such as the parallelization *bounds*.

prelude! {
    conf::ComponentPara,
}

pub mod generic;

pub type Weight = usize;

/// Gives values to weight bounds and instruction weight.
#[allow(non_upper_case_globals)]
pub mod weight {
    use super::Weight;

    macro_rules! weights {
        { $( $(#[$meta:meta])* $id:ident = $val:expr ),* $(,)? } => {
            $( $(#[$meta])* pub const $id: Weight = $val ; )*
        };
    }

    weights! {
        /// Weight under which no parallelization takes place, *i.e.* sequential codegen.
        no_para_ubx = 10,
        /// Weight under which we use rayon.
        rayon_ubx = 100,
        /// Weight under which we use threads, no parallelization for higher weights.
        threads_ubx = 10_000,

        /// Lowest weight above [weight::no_para_ubx], *i.e.* rayon's lower bound (inclusive).
        rayon_lbi = no_para_ubx,
        /// Lowest weight above [weight::rayon_ubx], *i.e.* threads' lower bound (inclusive).
        threads_lbi = rayon_ubx,
        /// Lowest weight above [weight::threads_ubx], *i.e.* "infinite weight"'s lower bound
        /// (inclusive).
        infinity_lbi = threads_ubx,

        /// "Zero" (sub-)expression weight.
        zero = 0,
        /// "Low" (sub-)expression weight.
        lo = 1,
        /// "Medium" (sub-)expression weight.
        mid = 4,
        /// "High" (sub-)expression weight.
        hi = 7,
    }

    /// Turns a usize into a [Weight].
    pub fn from_usize(n: usize) -> Weight {
        n
    }
}

/// Computes the weight of a collection or an option.
#[macro_export]
macro_rules! w8 {
    { weight $e:expr, $($map_fn:tt)* } => { $e.iter().map($($map_fn)*) };
    { $wb:expr, $ctx:expr $(,)? => weight $e:expr } => {
        $crate::w8!(weight $e, |e| e.weight($wb, $ctx))
    };
    { weight? $e:expr, $($map_fn:tt)* } => {
        $e.map($($map_fn)*).unwrap_or($crate::synced::weight::zero)
    };
    { $wb:expr, $ctx:expr $(,)? => weight? $e:expr } => {
        $crate::w8!(weight? $e, |e| e.weight($wb, $ctx))
    };
    { $id:ident $e:expr, $($map_fn:tt)* } => {
        $crate::w8!(weight $e, $($map_fn)*).$id::<$crate::synced::Weight>()
    };
    { $wb:expr, $ctx:expr $(,)? => $id:ident $e:expr } => {
        $crate::w8!($id $e, |e| e.weight($wb, $ctx))
    };
}

/// *Weight bounds* define weight intervals that correspond to parallelization strategies.
///
/// # Intervals
///
/// | lower bound, inclusive | upper bound, exclusive |      [Kind]       |
/// | :--------------------: | :--------------------: | :---------------: |
/// |          `0`           |  [Self::no_para_ubx]   |    [Kind::Seq]    |
/// |  [Self::no_para_ubx]   |   [Self::rayon_ubx]    | [Kind::FastRayon] |
/// |   [Self::rayon_ubx]    |  [Self::threads_ubx]   |  [Kind::Threads]  |
/// |  [Self::threads_ubx]   |          `∞`           |    [Kind::Seq]    |
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WeightBounds {
    /// Exclusive upper-bound for sequential codegen.
    no_para_ubx: Weight,
    /// Exclusive upper-bound for rayon codegen.
    rayon_ubx: Weight,
    /// Exclusive upper-bound for threads codegen.
    threads_ubx: Weight,
}
impl WeightBounds {
    /// Proc-macro parser.
    ///
    /// Parses either
    /// - nothing, yields the default bounds;
    /// - `(<usize>, <usize>, <usize>)` used as values for the [WeightBounds].
    pub fn parse(input: ParseStream) -> syn::Res<Self> {
        if input.peek(syn::token::Paren) {
            let content;
            let parens = parenthesized!(content in input);
            macro_rules! parse_val {{} => {{
                let val: syn::LitInt = content.parse()?;
                val.base10_parse()?
            }}}
            let no_para_ubx = parse_val!();
            let _: Token![,] = content.parse()?;
            let rayon_ubx = parse_val!();
            let _: Token![,] = content.parse()?;
            let threads_ubx = parse_val!();
            if content.peek(Token![,]) {
                let _: Token![,] = content.parse()?;
            }
            if !content.is_empty() {
                return Err(syn::Error::new(content.span(), "expected nothing here"));
            }
            if rayon_ubx < no_para_ubx {
                return Err(syn::Error::new(
                    parens.span.close(),
                    format!(
                        "illegal rayon weight upper-bound: lower than lower-bound, `{} < {}`",
                        rayon_ubx, no_para_ubx,
                    ),
                ));
            }
            if threads_ubx < rayon_ubx {
                return Err(syn::Error::new(
                    parens.span.close(),
                    format!(
                        "illegal threads weight upper-bound: lower than lower-bound, `{} < {}`",
                        threads_ubx, rayon_ubx,
                    ),
                ));
            }
            Ok(Self {
                no_para_ubx,
                rayon_ubx,
                threads_ubx,
            })
        } else {
            Ok(Self::default())
        }
    }

    /// Default weight bounds.
    pub const DEFAULT: Self = Self {
        no_para_ubx: weight::no_para_ubx,
        rayon_ubx: weight::rayon_ubx,
        threads_ubx: weight::threads_ubx,
    };

    /// Constructor.
    pub fn new(no_para_ubx: Weight, rayon_ubx: Weight, threads_ubx: Weight) -> Self {
        Self {
            no_para_ubx,
            rayon_ubx,
            threads_ubx,
        }
    }

    /// Only allows rayon with a *coefficient* `≠ 0`.
    ///
    /// The higher the *coefficient*, the **more expensive** statements must be to be rayon-ized.
    pub fn only_rayon_mult(n: usize) -> Self {
        debug_assert_ne!(n, 0);
        Self::new(
            Self::DEFAULT.no_para_ubx,
            Self::DEFAULT.rayon_ubx * n,
            Self::DEFAULT.rayon_ubx * n,
        )
    }
    /// Only allows rayon.
    pub fn only_rayon() -> Self {
        Self::only_rayon_mult(1)
    }

    /// Only allows rayon with a *coefficient* `≠ 0`.
    ///
    /// The higher the *coefficient*, the **cheaper** statements can be to be threads-ized.
    pub fn only_threads_div(n: usize) -> Self {
        debug_assert_ne!(n, 0);
        Self::new(
            Self::DEFAULT.no_para_ubx / n,
            Self::DEFAULT.no_para_ubx / n,
            Self::DEFAULT.threads_ubx,
        )
    }
    /// Only allows threads.
    pub fn only_threads() -> Self {
        Self::only_threads_div(1)
    }
    /// Allows threads and rayon.
    pub fn mixed() -> Self {
        Self::default()
    }

    pub fn no_para_ubx(&self) -> Weight {
        self.no_para_ubx
    }
    pub fn rayon_ubx(&self) -> Weight {
        self.rayon_ubx
    }
    pub fn threads_ubx(&self) -> Weight {
        self.threads_ubx
    }

    /// True if the bounds allow rayon.
    pub fn has_rayon(&self) -> bool {
        self.no_para_ubx < self.rayon_ubx
    }
    /// True if the bounds allow threads.
    pub fn has_threads(&self) -> bool {
        self.rayon_ubx < self.threads_ubx
    }

    /// The weight of an external function call given an optional weight hint in percent.
    ///
    /// How this function works is arbitrary, pretty much.
    ///
    /// First, we can't really do anything if we don't have weight hint so in this case the
    /// convention is "do nothing when given nothing", and we don't parallelize.
    ///
    /// If we have a weight hint then we apply the following cutoffs:
    ///
    /// - `000% ≤ hint < 010%`: don't parallelize (so no hint is basically the same as `hint = 0`);
    /// - `010% ≤ hint < 030%`: use rayon;
    /// - `030% ≤ hint < 100%`: use threads;
    /// - `100% ≤ hint`: don't parallelize (too expensive).
    ///
    /// Currently the weight returned is **not** proportional to the `hint` precise value in its
    /// interval.
    ///
    /// Note that extreme cases this function may not behave as discussed above, for instance if
    /// some bounds in `self` are equal or `no_para_ubx` is `0`.
    pub fn function_weight(&self, weight_percent_hint: Option<usize>) -> usize {
        if let Some(hint) = weight_percent_hint {
            if hint < 10 {
                if 0 < self.no_para_ubx {
                    self.no_para_ubx - 1
                } else {
                    0
                }
            } else if hint < 30 {
                let diff = self.rayon_ubx - self.no_para_ubx;
                self.no_para_ubx + (diff / 2)
            } else if hint < 100 {
                let diff = self.threads_ubx - self.rayon_ubx;
                self.rayon_ubx + (diff / 2)
            } else {
                self.threads_ubx
            }
        } else {
            0
        }
    }

    /// Returns the [Kind] corresponding to a [Weight], see [WeightBounds] for the intervals.
    pub fn decide(&self, weight: Weight) -> Kind {
        if weight < self.no_para_ubx {
            Kind::Seq
        } else if weight < self.rayon_ubx {
            Kind::FastRayon
        } else if weight < self.threads_ubx {
            Kind::Threads
        } else {
            Kind::Seq
        }
    }
}
impl Default for WeightBounds {
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

/// Distinguishes between parallelization kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    /// No parallelization, sequential.
    Seq,
    /// Fast rayon using [Option]s.
    FastRayon,
    /// Normal, modern rayon.
    Rayon,
    /// Threads using thread-scopes.
    Threads,
}
impl Kind {
    /// Decides on how to parallelize some statements given a para-conf and a weight.
    pub fn decide(para_conf: &ComponentPara, weight: Weight) -> Kind {
        match para_conf {
            ComponentPara::None => Self::Seq,
            ComponentPara::Para(bounds) => bounds.decide(weight),
        }
    }
}
impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Seq => "seq".fmt(f),
            Self::FastRayon => "fast-rayon".fmt(f),
            Self::Rayon => "rayon".fmt(f),
            Self::Threads => "threads".fmt(f),
        }
    }
}
