//! Extensions for types defined outside of this crate.

prelude! {}

pub trait ItemImplExt {
    type Name;
    type Itm;
    fn new_simple(name: Self::Name, items: Self::Itm) -> Self;
}
impl ItemImplExt for syn::ItemImpl {
    type Name = syn::Type;
    type Itm = Vec<syn::ImplItem>;
    fn new_simple(ty: syn::Type, items: Vec<syn::ImplItem>) -> Self {
        syn::ItemImpl {
            attrs: Vec::with_capacity(0),
            defaultness: None,
            unsafety: None,
            impl_token: parse_quote!(impl),
            generics: parse_quote!(),
            trait_: None,
            self_ty: ty.into(),
            brace_token: Default::default(),
            items,
        }
    }
}
impl ItemImplExt for syn::ItemMod {
    type Name = syn::Ident;
    type Itm = Vec<syn::Item>;
    fn new_simple(ident: syn::Ident, items: Vec<syn::Item>) -> Self {
        syn::ItemMod {
            attrs: Vec::with_capacity(0),
            vis: parse_quote!(pub),
            unsafety: None,
            mod_token: parse_quote!(mod),
            ident,
            content: Some((Default::default(), items)),
            semi: None,
        }
    }
}
impl ItemImplExt for syn::Arm {
    type Name = syn::Pat;
    type Itm = syn::Expr;
    fn new_simple(pat: syn::Pat, expr: syn::Expr) -> Self {
        syn::Arm {
            attrs: Vec::with_capacity(0),
            pat,
            guard: None,
            fat_arrow_token: parse_quote!(=>),
            body: Box::new(expr),
            comma: parse_quote!(,),
        }
    }
}
impl ItemImplExt for syn::Block {
    type Name = ();
    type Itm = Vec<syn::Stmt>;
    fn new_simple(_: (), stmts: Vec<syn::Stmt>) -> Self {
        syn::Block {
            brace_token: Default::default(),
            stmts,
        }
    }
}
impl ItemImplExt for syn::ExprBlock {
    type Name = ();
    type Itm = Vec<syn::Stmt>;
    fn new_simple(_: (), stmts: Vec<syn::Stmt>) -> Self {
        syn::ExprBlock {
            attrs: Vec::with_capacity(0),
            label: None,
            block: syn::Block {
                brace_token: Default::default(),
                stmts,
            },
        }
    }
}
impl ItemImplExt for syn::ExprMatch {
    type Name = syn::Expr;
    type Itm = Vec<syn::Arm>;
    fn new_simple(e: syn::Expr, arms: Vec<syn::Arm>) -> Self {
        syn::ExprMatch {
            attrs: Vec::with_capacity(0),
            match_token: parse_quote!(match),
            expr: Box::new(e),
            brace_token: Default::default(),
            arms,
        }
    }
}
impl ItemImplExt for syn::ExprIf {
    type Name = syn::Expr;
    type Itm = (syn::Block, Option<syn::Expr>);
    fn new_simple(cond: syn::Expr, (then_branch, els): Self::Itm) -> Self {
        let else_branch = els.map(|e| (Default::default(), Box::new(e)));
        syn::ExprIf {
            attrs: Vec::with_capacity(0),
            if_token: parse_quote!(if),
            cond: Box::new(cond),
            then_branch,
            else_branch,
        }
    }
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

pub trait IdentExt: Sized {
    fn to_snake_with(&self, pref: impl AsRef<str>, suff: impl AsRef<str>) -> Self;
    fn to_camel_with(&self, pref: impl AsRef<str>, suff: impl AsRef<str>) -> Self;

    fn instant_var() -> Self;
    fn init_instant_var() -> Self;
    fn result(span: Span) -> Self;

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
    fn result(span: Span) -> Self {
        Ident::new("result", span)
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

pub trait MetaExt: Sized {
    fn parse_grust_meta<T>(
        self,
        f_path: impl FnOnce(syn::Path) -> syn::Res<T>,
        f_meta_list: impl FnOnce(syn::Path, TokenStream2) -> syn::Res<T>,
        f_value: impl FnOnce(syn::Path, syn::Expr) -> syn::Res<T>,
    ) -> syn::Res<T>;
    fn parse_grust_ident_value<T>(
        self,
        f: impl FnOnce(&syn::Ident, syn::Expr) -> syn::Res<T>,
    ) -> syn::Res<T> {
        self.parse_grust_meta(
            |p| Err(syn::Error::new_spanned(p, "unexpected path attribute")),
            |p, _| {
                Err(syn::Error::new_spanned(
                    p,
                    "unexpected metal-list attribute",
                ))
            },
            |p, e| {
                let name = if let Some(id) = p.get_ident() {
                    id
                } else {
                    return Err(syn::Error::new_spanned(p, "expected identifier"));
                };
                f(name, e)
            },
        )
    }

    fn parse_weight_percent_hint(self) -> syn::Res<Option<usize>> {
        self.parse_grust_ident_value(|id, e| {
            let w = if let "weight_percent" = id.to_string().as_str() {
                Some(e.parse_usize()?)
            } else {
                None
            };
            Ok(w)
        })
    }
}
impl MetaExt for syn::Meta {
    fn parse_grust_meta<T>(
        self,
        f_path: impl FnOnce(syn::Path) -> syn::Res<T>,
        f_meta_list: impl FnOnce(syn::Path, TokenStream2) -> syn::Res<T>,
        f_value: impl FnOnce(syn::Path, syn::Expr) -> syn::Res<T>,
    ) -> syn::Res<T> {
        match self {
            Self::Path(path) => f_path(path),
            Self::List(ml) => f_meta_list(ml.path, ml.tokens),
            Self::NameValue(nv) => f_value(nv.path, nv.value),
        }
    }
}

pub trait SynExprExt {
    fn parse_usize(&self) -> syn::Res<usize>;
}
impl SynExprExt for syn::Expr {
    fn parse_usize(&self) -> syn::Res<usize> {
        if let syn::Expr::Lit(lit) = self {
            if let syn::Lit::Int(n) = &lit.lit {
                return n.base10_parse();
            }
        }
        Err(syn::Error::new_spanned(self, "expected a `usize` value"))
    }
}
