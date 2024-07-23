use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use petgraph::graphmap::{DiGraphMap, NodeTrait};
use serde::Serialize;

/// To use with serde's `serialize_with` attribute.
pub fn ordered_graph<S, K, V>(value: &DiGraphMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    K: NodeTrait + Hash + Clone + Copy + Ord + serde::Serialize,
    V: serde::Serialize,
{
    value
        .all_edges()
        .map(|(a, b, c)| ((a, b), c))
        .collect::<BTreeMap<_, _>>()
        .serialize(serializer)
}

/// To use with serde's `serialize_with` attribute.
pub fn ordered_hashmap<S, K, V, HMS>(
    value: &HashMap<K, V, HMS>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    K: Hash + Clone + Copy + Ord + serde::Serialize,
    V: serde::Serialize,
{
    value
        .iter()
        .collect::<BTreeMap<_, _>>()
        .serialize(serializer)
}
