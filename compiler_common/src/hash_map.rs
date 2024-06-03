//! Custom hash-map/-set with a decidable key ordering.

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

/// An alias for a hashset using `twox_hash::XxHash64`.
pub type HashSet<K> = rustc_hash::FxHashSet<K>;

pub trait HashSetNew {
    fn new() -> Self;
    fn with_capacity(capacity: usize) -> Self;
}

impl<V> HashSetNew for HashSet<V> {
    fn new() -> Self {
        Default::default()
    }
    fn with_capacity(capacity: usize) -> Self {
        HashSet::with_capacity_and_hasher(capacity, Default::default())
    }
}
