//! Parallelization stuff.

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
        threads_ubx = 100_00,

        /// "Zero" (sub-)expression weight.
        zero = 0,
        /// "Low" (sub-)expression weight.
        lo = 1,
        /// "Medium" (sub-)expression weight.
        mid = 2,
        /// "High" (sub-)expression weight.
        hi = 3,
    }

    /// Turns a usize into a [Weight].
    pub fn from_usize(n: usize) -> Weight {
        n
    }
}

/// Can compute its [Weight].
pub trait HasWeight {
    /// Self's weight.
    fn weight(&self, bounds: &WeightBounds) -> Weight;
}
impl HasWeight for Weight {
    fn weight(&self, _: &WeightBounds) -> Weight {
        *self
    }
}

/// Computes the weight of a collection or an option.
#[macro_export]
macro_rules! w8 {
    { weight $e:expr, $($map_fn:tt)* } => { $e.iter().map($($map_fn)*) };
    { $wb:expr => weight $e:expr  } => { $crate::w8!(weight $e, |e| e.weight($wb)) };
    { weight? $e:expr, $($map_fn:tt)* } => {
        $e.map($($map_fn)*).unwrap_or($crate::synced::weight::zero)
    };
    { $wb:expr => weight? $e:expr  } => { $crate::w8!(weight? $e, |e| e.weight($wb)) };
    { $id:ident $e:expr, $($map_fn:tt)* } => {
        $crate::w8!(weight $e, $($map_fn)*).$id::<$crate::synced::Weight>()
    };
    { $wb:expr => $id:ident $e:expr } => { $crate::w8!($id $e, |e| e.weight($wb)) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WeightBounds {
    no_para_ubx: Weight,
    rayon_ubx: Weight,
    threads_ubx: Weight,
}
impl WeightBounds {
    /// Proc-macro parser.
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

    pub const DEFAULT: Self = Self {
        no_para_ubx: 100,
        rayon_ubx: 200,
        threads_ubx: 10_000,
    };

    fn new(no_para_ubx: Weight, rayon_ubx: Weight, threads_ubx: Weight) -> Self {
        Self {
            no_para_ubx,
            rayon_ubx,
            threads_ubx,
        }
    }

    pub fn only_rayon_mult(n: usize) -> Self {
        debug_assert_ne!(n, 0);
        Self::new(
            Self::DEFAULT.no_para_ubx,
            Self::DEFAULT.rayon_ubx * n,
            Self::DEFAULT.rayon_ubx * n,
        )
    }
    pub fn only_rayon() -> Self {
        Self::only_rayon_mult(1)
    }
    pub fn only_threads_div(n: usize) -> Self {
        debug_assert_ne!(n, 0);
        Self::new(
            Self::DEFAULT.no_para_ubx / n,
            Self::DEFAULT.no_para_ubx / n,
            Self::DEFAULT.threads_ubx,
        )
    }
    pub fn only_threads() -> Self {
        Self::only_threads_div(1)
    }
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

    pub fn has_rayon(&self) -> bool {
        self.no_para_ubx < self.rayon_ubx
    }
    pub fn has_threads(&self) -> bool {
        self.rayon_ubx < self.threads_ubx
    }

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
