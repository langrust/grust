//! Extensions for types defined outside of this crate.

prelude! {}

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

pub trait IdentExt: Sized {
    fn to_snake_with(&self, pref: impl AsRef<str>, suff: impl AsRef<str>) -> Self;
    fn to_camel_with(&self, pref: impl AsRef<str>, suff: impl AsRef<str>) -> Self;

    fn instant_var() -> Self;
    fn init_instant_var() -> Self;

    fn to_snake_pref(&self, pref: impl AsRef<str>) -> Self {
        self.to_snake_with(pref, "")
    }
    fn to_snake_suff(&self, suff: impl AsRef<str>) -> Self {
        self.to_snake_with("", suff)
    }
    fn to_snake(&self) -> Self {
        self.to_snake_with("", "")
    }

    fn to_camel_pref(&self, pref: impl AsRef<str>) -> Self {
        self.to_camel_with(pref, "")
    }
    fn to_camel_suff(&self, suff: impl AsRef<str>) -> Self {
        self.to_camel_with("", suff)
    }
    fn to_camel(&self) -> Self {
        self.to_camel_with("", "")
    }

    /// Alias for [IdentExt::to_snake].
    fn to_field(&self) -> Self {
        self.to_snake()
    }
    /// Alias for [IdentExt::to_snake].
    fn to_var(&self) -> Self {
        self.to_snake()
    }
    /// Alias for [IdentExt::to_camel].
    fn to_ty(&self) -> Self {
        self.to_camel()
    }

    /// Adapts the identifier to one for a state type, typically for a component/state-machine.
    fn to_state_ty(&self) -> Self {
        self.to_camel_suff("State")
    }
    /// Adapts the identifier to one for an input type, typically for a component/state-machine.
    fn to_input_ty(&self) -> Self {
        self.to_camel_suff("Input")
    }

    /// Adapts the identifier to one for a service store type.
    fn to_service_store_ty(&self) -> Self {
        self.to_camel_suff("ServiceStore")
    }
    /// Adapts the identifier to one for a service state type.
    fn to_service_state_ty(&self) -> Self {
        self.to_camel_suff("Service")
    }
    /// Adapts the identifier to one for a service module.
    fn to_service_mod(&self) -> Self {
        self.to_snake_suff("_service")
    }

    fn to_last_var(&self) -> Self {
        self.to_snake_pref("last_")
    }
    fn to_instant_var(&self) -> Self {
        self.to_snake_with("_", "_instant")
    }
    fn to_ref_var(&self) -> Self {
        self.to_snake_suff("_ref")
    }
    fn to_handle_fn(&self) -> Self {
        self.to_snake_pref("handle_")
    }
}

impl IdentExt for Ident {
    fn instant_var() -> Self {
        Ident::new("_grust_reserved_instant", Span::mixed_site())
    }
    fn init_instant_var() -> Self {
        Ident::new("_grust_reserved_init_instant", Span::mixed_site())
    }

    fn to_camel_with(&self, pref: impl AsRef<str>, suff: impl AsRef<str>) -> Self {
        Self::new(
            &format!(
                "{}{}{}",
                pref.as_ref(),
                to_camel_case(self.to_string()),
                suff.as_ref()
            ),
            self.span(),
        )
    }
    fn to_snake_with(&self, pref: impl AsRef<str>, suff: impl AsRef<str>) -> Self {
        Self::new(
            &format!(
                "{}{}{}",
                pref.as_ref(),
                to_snake_case(self.to_string()),
                suff.as_ref()
            ),
            self.span(),
        )
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
