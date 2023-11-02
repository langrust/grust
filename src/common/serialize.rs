use std::collections::{BTreeMap, HashMap};

/// To use with serde's [serialize_with] attribute.
pub fn ordered_map<S, K: Ord + serde::Serialize, V: serde::Serialize>(
    value: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    serde::Serialize::serialize(&ordered, serializer)
}
