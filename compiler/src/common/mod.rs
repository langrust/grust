/// Imports the compiler prelude.
///
/// Can be customized by passing import paths **relative to the root of the crate**:
///
/// ```rust
/// # use compiler::prelude;
/// prelude! { ast::Item }
/// pub type I = Item;
/// ```
///
/// This includes:
///
/// - `crate::utils`
#[macro_export]
macro_rules! prelude {
    { $($imports:tt)* } => {
        #[allow(unused_imports)]
        use $crate::{common::hash_map::*, $($imports)*};
    }
}

/// Location handler module.
pub mod location;

/// Type system module.
pub mod r#type;

/// Constant module.
pub mod constant;

/// Operator module.
pub mod operator;

/// Scope module.
pub mod scope;

/// Serialization module.
pub mod serialize;

/// Case converter module.
pub mod convert_case;

/// Graph label module.
pub mod label;

/// Graph color module.
pub mod color;

pub mod hash_map {
    /// An alias for a hashmap using `twox_hash::XxHash64`.
    pub type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

    pub trait HashMapNew {
        fn new() -> Self;
        fn with_capacity(capacity: usize) -> Self;
    }

    impl<K, V> HashMapNew for HashMap<K, V> {
        fn new() -> Self {
            Default::default()
        }
        fn with_capacity(capacity: usize) -> Self {
            HashMap::with_capacity_and_hasher(capacity, Default::default())
        }
    }
}
