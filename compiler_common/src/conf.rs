prelude! {}

/// Services configuration for the propagation of
/// events and signals changes.
#[derive(Clone, Default)]
pub enum Propagation {
    #[default]
    EventIsles,
    OnChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentPara {
    None,
    Rayon1,
    Rayon2,
    Rayon3,
    Threads,
    Mixed,
}
impl Default for ComponentPara {
    fn default() -> Self {
        Self::None
    }
}
impl ComponentPara {
    pub fn is_none(self) -> bool {
        match self {
            Self::None => true,
            Self::Rayon1 | Self::Rayon2 | Self::Rayon3 | Self::Threads | Self::Mixed => false,
        }
    }
    pub fn is_rayon(self, cnd: bool) -> bool {
        match self {
            Self::None | Self::Threads => false,
            Self::Rayon1 | Self::Rayon2 | Self::Rayon3 => true,
            Self::Mixed => cnd,
        }
    }
    pub fn has_threads(self) -> bool {
        match self {
            Self::Threads | Self::Mixed => true,
            Self::None | Self::Rayon1 | Self::Rayon2 | Self::Rayon3 => false,
        }
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
        para: bool = false =>
            /// Item for the `para` configuration value.
            Para,
        component_para: ComponentPara = ComponentPara::default() =>
            /// Item for the `component_para` configuration value.
            ComponentPara,
        pub_components: bool = false =>
            /// Item for the `pub_components` configuration value.
            PubComponent,
        dump_code: Option<syn::LitStr> = None =>
            /// Item for the `dump_code` configuration value.
            DumpCode,
        greusot: bool = false =>
            /// Item for the `greusot` configuration value.
            Greusot,
        test: bool = false =>
            /// Item for the `test` configuration value.
            Test,
        demo: bool = false =>
            /// Item for the `demo` configuration value.
            Demo,
        stats_depth: usize = 0 =>
            /// Item for the `stats_depth` configuration value.
            StatsDepth,
    }
}

impl Conf {
    pub fn check_sanity(&self, report_on: Span) -> syn::Res<()> {
        if self.greusot && (self.test || self.demo) {
            return Err(syn::Error::new(
                report_on,
                "illegal configuration: `greusot` cannot be active in `test` or `demo` modes",
            ));
        }
        if self.test && self.demo {
            return Err(syn::Error::new(
                report_on,
                "illegal configuration: `test` and `demo` modes are incompatible",
            ));
        }
        Ok(())
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
                    let val: syn::LitStr = input.parse()?;
                    match val.value().as_str() {
                        "onchange" => Self::Propagation(span, Propagation::OnChange),
                        "onevent" => Self::Propagation(span, Propagation::EventIsles),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                val,
                                "unexpected propagation configuration, \
                                expected `onchange` or `onevent`",
                            ));
                        }
                    }
                }
                "dump" => {
                    let _: Token![=] = input.parse()?;
                    let val: syn::LitStr = input.parse()?;
                    Self::DumpCode(span, Some(val))
                }
                "stats_depth" => {
                    let _: Token![=] = input.parse()?;
                    let val: syn::LitInt = input.parse()?;
                    let val: usize = val.base10_parse()?;
                    Self::StatsDepth(span, val)
                }
                "para" => Self::Para(span, true),
                "component_para_none" => Self::ComponentPara(span, ComponentPara::None),
                "component_para_threads" => Self::ComponentPara(span, ComponentPara::Threads),
                "component_para_rayon1" => Self::ComponentPara(span, ComponentPara::Rayon1),
                "component_para_rayon2" => Self::ComponentPara(span, ComponentPara::Rayon2),
                "component_para_rayon3" => Self::ComponentPara(span, ComponentPara::Rayon3),
                "component_para_mixed" => Self::ComponentPara(span, ComponentPara::Mixed),
                "pub" => Self::PubComponent(span, true),
                "greusot" => Self::Greusot(span, true),
                "test" => Self::Test(span, true),
                "demo" => Self::Test(span, true),
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
                    let span = item.span();
                    slf.with(item);
                    slf.check_sanity(span)?;
                }
            }
            Ok(slf)
        }
    }
}
