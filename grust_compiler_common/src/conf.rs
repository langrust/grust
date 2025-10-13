//! Compiler configuration elements.

prelude! {
    synced::WeightBounds,
}

/// Services configuration for the propagation of
/// events and signals changes.
#[derive(Clone, Default)]
pub enum Propagation {
    EventIsles,
    #[default]
    OnChange,
}

/// Component parallelization configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentPara {
    None,
    Para(WeightBounds),
}
impl From<WeightBounds> for ComponentPara {
    fn from(val: WeightBounds) -> Self {
        ComponentPara::Para(val)
    }
}
impl Default for ComponentPara {
    fn default() -> Self {
        Self::None
    }
}
impl ComponentPara {
    pub fn none() -> Self {
        Self::None
    }
    pub fn para(no_para_ubx: usize, rayon_ubx: usize, threads_ubx: usize) -> Self {
        Self::Para(WeightBounds::new(no_para_ubx, rayon_ubx, threads_ubx))
    }
    pub fn rayon_mult(n: usize) -> Self {
        WeightBounds::only_rayon_mult(n).into()
    }
    pub fn rayon() -> Self {
        WeightBounds::only_rayon().into()
    }
    pub fn threads_div(n: usize) -> Self {
        WeightBounds::only_threads_div(n).into()
    }
    pub fn threads() -> Self {
        WeightBounds::only_threads().into()
    }
    pub fn mixed() -> Self {
        WeightBounds::mixed().into()
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn decide(&self, weight: synced::Weight) -> synced::Kind {
        synced::Kind::decide(self, weight)
    }
}

/// Compilation modes.
#[derive(Clone, Default)]
pub enum CompilationMode {
    #[default]
    Demo,
    Greusot,
    Test,
}
impl CompilationMode {
    /// Tells if the compilation mode is GReusot or not.
    pub fn greusot(&self) -> bool {
        if let Self::Greusot = self {
            return true;
        }
        false
    }
    /// Tells if the compilation mode is Test or not.
    pub fn test(&self) -> bool {
        if let Self::Test = self {
            return true;
        }
        false
    }
    /// Tells if the compilation mode is Demo or not.
    pub fn demo(&self) -> bool {
        if let Self::Demo = self {
            return true;
        }
        false
    }
}

macro_rules! build_conf {
    {
        $(#[$conf_meta:meta])*
        $conf_struct:ident where Item = $conf_item_enum:ident {
            $(
                $(#[$field_meta:meta])*
                $field_id:ident
                :
                $field_ty:ty
                =
                $field_default:expr
                =>
                $(#[$field_variant_meta:meta])*
                $field_variant:ident
            ),* $(,)?
        }
    } => {
        // conf `struct`, all fields public
        $(#[$conf_meta])*
        pub struct $conf_struct {
            $(
                $(#[$field_meta])*
                pub $field_id : $field_ty,
            )*
        }
        // `Default` implementation
        impl std::default::Default for $conf_struct {
            fn default() -> Self {
                Self {
                    $( $field_id: $field_default, )*
                }
            }
        }
        // config item enumeration
        /// Enumeration of all the configuration items.
        pub enum $conf_item_enum {
            $(
                $(#[$field_variant_meta])*
                $field_variant ( $crate::prelude::Span, $field_ty ),
            )*
        }
        impl $conf_item_enum {
            /// Span to report errors on this item on.
            pub fn span(&self) -> Span {
                match self {
                    $( Self::$field_variant(span, _) => span.clone(), )*
                }
            }
        }

        impl $conf_struct {
            /// Updates a configuration value.
            pub fn with(&mut self, item: $conf_item_enum) {
                match item {
                    $(
                        $conf_item_enum::$field_variant(_, data) => self.$field_id = data,
                    )*
                }
            }
        }
    };
}

build_conf! {
    /// Compiler configuration.
    Conf where Item = ConfItem {
        propagation: Propagation = Propagation::default() =>
            /// Item for the `propagation` configuration value.
            Propagation,
        service_para: bool = false =>
            /// Item for the `service_para` configuration value.
            ServicePara,
        component_para: ComponentPara = ComponentPara::default() =>
            /// Item for the `component_para` configuration value.
            ComponentPara,
        align: bool = false =>
            /// Item for the `align` configuration value.
            AlignMem,
        public: bool = true =>
            /// Item for the `public` configuration value.
            PubThings,
        dump_code: Option<syn::LitStr> = None =>
            /// Item for the `dump_code` configuration value.
            DumpCode,
        dump_graph: Option<syn::LitStr> = None =>
            /// Item for the `dump_graph` configuration value.
            DumpGraph,
        mode: CompilationMode = CompilationMode::default() =>
            /// Item for the `mode` configuration value.
            Mode,
        levenshtein: bool = false =>
            /// Item for the `levenshtein` configuration value.
            Levenshtein,
        stats_depth: usize = 0 =>
            /// Item for the `stats_depth` configuration value.
            StatsDepth,
        spawn_with: Option<syn::Path> = None =>
            /// Item for the `dump_code` configuration value.
            SpawnWith,
        get_handle: Option<syn::Type> = None =>
            /// Item for the `dump_code` configuration value.
            GetHandle,
        tracing: bool = false =>
            /// Item for the `tracing` configuration value.
            Tracing,
    }
}

mod parsing {
    use super::*;

    impl syn::Parse for ConfItem {
        fn parse(input: ParseStream) -> syn::Res<Self> {
            let ident: Ident = input.parse()?;
            let span = ident.span();
            let item = match ident.to_string().as_str() {
                "propag" => {
                    let _: Token![=] = input.parse()?;
                    let ident: Ident = input.parse()?;
                    match ident.to_string().as_str() {
                        "onchange" => Self::Propagation(span, Propagation::OnChange),
                        "onevent" => Self::Propagation(span, Propagation::EventIsles),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                ident,
                                "unexpected propagation configuration, \
                                expected `onchange` or `onevent`",
                            ));
                        }
                    }
                }
                "mode" => {
                    let _: Token![=] = input.parse()?;
                    let ident: Ident = input.parse()?;
                    match ident.to_string().as_str() {
                        "greusot" => Self::Mode(span, CompilationMode::Greusot),
                        "test" => Self::Mode(span, CompilationMode::Test),
                        "demo" => Self::Mode(span, CompilationMode::Demo),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                ident,
                                "unexpected compilation mode, \
                                expected `demo`, `test` or `greusot`",
                            ));
                        }
                    }
                }
                "dump" => {
                    let _: Token![=] = input.parse()?;
                    let val: syn::LitStr = input.parse()?;
                    Self::DumpCode(span, Some(val))
                }
                "dump_graph" => {
                    let _: Token![=] = input.parse()?;
                    let val: syn::LitStr = input.parse()?;
                    Self::DumpGraph(span, Some(val))
                }
                "stats_depth" => {
                    let _: Token![=] = input.parse()?;
                    let val: syn::LitInt = input.parse()?;
                    let val: usize = val.base10_parse()?;
                    Self::StatsDepth(span, val)
                }
                "service_para" => Self::ServicePara(span, true),
                "align" => Self::AlignMem(span, true),
                "public" => {
                    let _: Token![=] = input.parse()?;
                    let val: syn::LitBool = input.parse()?;
                    let val: bool = val.value();
                    Self::PubThings(span, val)
                }
                "tracing" => Self::Tracing(span, true),
                "levenshtein" => {
                    let _: Token![=] = input.parse()?;
                    let val: syn::LitBool = input.parse()?;
                    let val: bool = val.value();
                    Self::Levenshtein(span, val)
                }
                "spawn_with" => {
                    let _: Token![=] = input.parse()?;
                    let str: syn::LitStr = input.parse()?;
                    let val: syn::Path = str.parse()?;
                    Self::SpawnWith(span, Some(val))
                }
                "get_handle" => {
                    let _: Token![=] = input.parse()?;
                    let str: syn::LitStr = input.parse()?;
                    let val: syn::Type = str.parse()?;
                    Self::GetHandle(span, Some(val))
                }
                "component_para" => Self::ComponentPara(span, WeightBounds::parse(input)?.into()),
                "component_para_none" => Self::ComponentPara(span, ComponentPara::none()),
                "component_para_threads" => Self::ComponentPara(span, ComponentPara::threads()),
                "component_para_threads1" => {
                    Self::ComponentPara(span, ComponentPara::threads_div(1))
                }
                "component_para_threads2" => {
                    Self::ComponentPara(span, ComponentPara::threads_div(2))
                }
                "component_para_threads3" => {
                    Self::ComponentPara(span, ComponentPara::threads_div(3))
                }
                "component_para_rayon" => Self::ComponentPara(span, ComponentPara::rayon()),
                "component_para_rayon1" => Self::ComponentPara(span, ComponentPara::rayon_mult(1)),
                "component_para_rayon2" => Self::ComponentPara(span, ComponentPara::rayon_mult(2)),
                "component_para_rayon3" => Self::ComponentPara(span, ComponentPara::rayon_mult(3)),
                "component_para_mixed" => Self::ComponentPara(span, ComponentPara::mixed()),
                _ => {
                    return Err(syn::Error::new_spanned(
                        ident,
                        "unexpected configuration key",
                    ))
                }
            };
            Ok(item)
        }
    }

    impl syn::Parse for Conf {
        fn parse(input: ParseStream) -> ::syn::Result<Self> {
            let mut slf = Self::default();
            if let Ok(true) = input
                .fork()
                .call(syn::Attribute::parse_inner)
                .map(|attrs| !attrs.is_empty())
            {
                let _: Token![#] = input.parse()?;
                let _: Token![!] = input.parse()?;
                let content;
                let _ = bracketed!(content in input);
                let items: syn::Punctuated<ConfItem, Token![,]> =
                    syn::Punctuated::parse_terminated(&content)?;
                for item in items {
                    slf.with(item);
                }
            }
            Ok(slf)
        }
    }
}
