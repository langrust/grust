use petgraph::graphmap::{DiGraphMap, NodeTrait};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use crate::hir::once_cell::OnceCell;

/// To use with serde's [serialize_with] attribute.
pub fn ordered_map<S, K, V>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    K: Ord + serde::Serialize,
    V: serde::Serialize,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    serde::Serialize::serialize(&ordered, serializer)
}

/// To use with serde's [serialize_with] attribute.
pub fn ordered_graph<S, K, V>(
    value: &OnceCell<DiGraphMap<K, V>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    K: NodeTrait + Hash + Clone + Copy + Ord + serde::Serialize,
    V: serde::Serialize,
{
    value
        .get()
        .map(|graph| {
            graph
                .all_edges()
                .map(|(a, b, c)| ((a, b), c))
                .collect::<BTreeMap<_, _>>()
        })
        .serialize(serializer)
}
