use std::sync::RwLock;

use lazy_static::lazy_static;

pub struct Conf {
    pub_nodes: bool,
    dump_code: Option<String>,
}
impl Default for Conf {
    fn default() -> Self {
        Self {
            pub_nodes: false,
            dump_code: None,
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
    bool {
        pub_nodes set_pub_nodes
    }
    Option<String> {
        dump_code set_dump_code
    }
}
