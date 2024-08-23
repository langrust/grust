pub use super::{
    new_graph,
    petgraph::{graphmap::DiGraphMap, visit::DfsEvent, Direction},
};
pub use {color::Color, label::Label};

mod color {
    /// [Color] enumeration used to identify the processing status of an element.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Color {
        /// Computation has ended.
        Black,
        /// Currently being processed.
        Grey,
        /// Element not processed.
        White,
    }

    impl Color {
        mk_new! {
            Black: black()
            Grey: grey()
            White: white()
        }
    }
}

mod label {
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    /// Edge label.
    pub enum Label {
        /// Contract label.
        Contract,
        /// Weighted label.
        Weight(usize),
    }

    mk_new! { impl Label =>
        Contract: contract()
        Weight: weight(n: usize = n)
    }

    impl Label {
        /// Add the two given labels.
        pub fn add(&self, other: &Label) -> Label {
            match (self, other) {
                (Label::Contract, _) => Label::Contract,
                (_, Label::Contract) => Label::Contract,
                (Label::Weight(w1), Label::Weight(w2)) => Label::Weight(w1 + w2),
            }
        }
        /// Increment the given label.
        pub fn increment(&self) -> Label {
            match self {
                Label::Contract => Label::Contract,
                Label::Weight(w) => Label::Weight(w + 1),
            }
        }
    }
}

/// Add an edge to a graph.
///
/// If a similar edge already exits then keep the edge with the lowest weight.
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

/// DSL for creating [`DiGraphMap`]-s, takes a list of potentially weighted edges.
///
/// # Format
///
/// - weightless edges: `<tt> -> <tt>`, for example
///   - `1 -> 3`
///   - `(get_src()) -> (val.tgt)`
///
/// - weighted edges: `<tt> -(<expr>)-> <tt>`, for example
///   - `1 -(7)-> 3`
///   - `(get_src()) -(3)-> (val.tgt)`
///
/// # Examples
///
/// Weightless edges, generates a [`DiGraphMap<_, ()>`]:
///
/// ```rust
/// # use compiler_common::new_graph;
/// let roots = [1, 3];
/// let graph = new_graph! {
///     (roots[0]) -> 2
///     2 -> 4
///     (roots[1]) -> 4
///     4 -> 5
/// };
/// let s = format!("{graph:?}");
/// assert_eq!(s, "{\
///     1: [(2, Outgoing)], \
///     2: [(1, Incoming), (4, Outgoing)], \
///     4: [(2, Incoming), (3, Incoming), (5, Outgoing)], \
///     3: [(4, Outgoing)], \
///     5: [(4, Incoming)]\
/// }");
/// ```
///
/// Weighted edges:
///
/// ```rust
/// # use compiler_common::{new_graph, graph::Direction};
/// let one_five = (1, 5);
/// let graph = new_graph! {
///     (one_five.0) -(3)-> 2
///     2 -(1)-> 4
///     3 -(2)-> 4
///     4 -(7)-> (one_five.1)
/// };
/// let s = format!("{graph:?}");
/// assert_eq!(s, "{\
///     1: [(2, Outgoing)], \
///     2: [(1, Incoming), (4, Outgoing)], \
///     4: [(2, Incoming), (3, Incoming), (5, Outgoing)], \
///     3: [(4, Outgoing)], \
///     5: [(4, Incoming)]\
/// }");
/// let edges: Vec<(usize, usize, &usize)> =
///     graph.edges_directed(4, Direction::Incoming).collect();
/// assert_eq!(edges, vec![(2, 4, &1), (3, 4, &2)]);
/// ```
#[macro_export]
macro_rules! new_graph {
    {
        $( $src:tt -> $tgt:tt )*
    } => {{
        let mut graph = $crate::graph::DiGraphMap::new();

        $(
            graph.add_edge($src, $tgt, ());
        )*

        graph
    }};
    {
        $( $src:tt -($weight:expr)-> $tgt:tt )*
    } => {{
        let mut graph = $crate::graph::DiGraphMap::new();

        $(
            graph.add_edge($src, $tgt, $weight);
        )*

        graph
    }};
}
