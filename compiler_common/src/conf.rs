use std::sync::RwLock;

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

/// Stores all possible compiler's configurations.
pub struct Conf {
    pub propagation: Propagation,
    pub para: bool,
    pub component_para: ComponentPara,
    pub pub_components: bool,
    pub dump_code: Option<String>,
    pub greusot: bool,
    pub test: bool,
    pub demo: bool,
    pub stats_depth: usize,
}
impl Default for Conf {
    fn default() -> Self {
        Self {
            propagation: Default::default(),
            para: false,
            component_para: Default::default(),
            pub_components: false,
            dump_code: None,
            greusot: false,
            test: false,
            demo: false,
            stats_depth: 0,
        }
    }
}

lazy_static! {
    /// Configuration.
    static ref CONF : RwLock<Conf> = RwLock::new(Conf::default());
}

fn read<T>(f: impl FnOnce(&Conf) -> T) -> T {
    let conf = CONF.read().expect("configuration lock is poisoned");
    f(&*conf)
}
fn write<T>(f: impl FnOnce(&mut Conf) -> T) -> T {
    let mut conf = CONF.write().expect("configuration lock is poisoned");
    f(&mut *conf)
}

/// Resets the global configuration to its default value.
///
/// The global state is maintained between proc-macro expansions, we need to reset before parsing a
/// new configuration to avoid errors such as "code-dump target already defined".
pub fn reset() {
    write(|conf| *conf = Conf::default())
}

macro_rules! def {
    { $(
        $typ:ty { $(
            $(#[$read_meta:meta])*
            $read_field:ident
            $(#[$write_meta:meta])*
            $write_field:ident
        )+ }
    )+ } => { $($(
        $(#[$read_meta])*
        pub fn $read_field() -> $typ {
            read(|conf| conf.$read_field.clone())
        }
        $(#[$write_meta])*
        pub fn $write_field(val : $typ) {
            write(|conf| conf.$read_field = val)
        }
    )* )+ };
}

def! {
    Propagation {
        #[doc = "Returns the propagation configuration."]
        propagation
        #[doc = "Set the propagation configuration."]
        set_propagation
    }
    bool {
        #[doc = "Tells if services' instructions are parallelized."]
        para
        #[doc = "Set parallelization in configuration."]
        set_para
    }
    ComponentPara {
        #[doc = "Specifies how to parallelize component code."]
        component_para
        #[doc = "Set component parallelization strategy."]
        set_component_para
    }
    bool {
        #[doc = "Tells if the components are public."]
        pub_components
        #[doc = "Set in configuration if the components are public."]
        set_pub_components
    }
    Option<String> {
        #[doc = "Returns `Some(path)` if the code should be written at `path`, \
        returns `None` if code should not be written."]
        dump_code
        #[doc = "Set in configuration where the code should be written."]
        set_dump_code
    }
    bool {
        #[doc = "Tells if greusot is used."]
        greusot
        #[doc = "Set in configuration if greusot is used."]
        set_greusot
    }
    bool {
        #[doc = "Tells if we are in test mode."]
        test
        #[doc = "Set test mode."]
        set_test
    }
    bool {
        #[doc = "Tells if we are in demo mode."]
        demo
        #[doc = "Set demo mode."]
        set_demo
    }
    usize {
        #[doc = "Stats printing depth, no output on `0`."]
        stats_depth
        #[doc = "Set `stats_depth`."]
        set_stats_depth
    }
}
