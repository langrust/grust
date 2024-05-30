/// Imports the compiler prelude.
///
/// Can be customized by passing import paths **relative to the root of the crate**:
///
/// ```rust
/// # use compiler::prelude;
/// prelude! { ast::Item };
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
        use crate::{common, $($imports)*};
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

pub type HMap<K, V> =
    std::collections::HashMap<K, V, std::hash::BuildHasherDefault<twox_hash::XxHash64>>;

pub fn new_hmap<K, V>() -> HMap<K, V> {
    Default::default()
}
