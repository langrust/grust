use petgraph::graphmap::DiGraphMap;

use crate::common::label::Label;

mod contract;
mod expression;
mod file;
mod node;
mod stream_expression;
mod statement;

/// Add an egde to the graph.
///
/// If edge already exits then keep the edge with the lowest weight.
pub fn add_edge(
    graph: &mut DiGraphMap<usize, Label>,
    signal_id: usize,
    dependency_id: usize,
    label: Label,
) {
    let prev_label = graph.add_edge(signal_id, dependency_id, label.clone());
    match (prev_label, label) {
        (Some(Label::Weight(prev_weight)), Label::Weight(new_weight))
            if prev_weight < new_weight =>
        {
            graph.add_edge(signal_id, dependency_id, Label::Weight(prev_weight));
        }
        _ => (),
    }
}
