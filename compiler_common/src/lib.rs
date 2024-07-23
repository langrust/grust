pub extern crate codespan_reporting;
pub extern crate itertools;
pub extern crate lazy_static;
pub extern crate once_cell;
pub extern crate petgraph;
pub extern crate proc_macro2 as macro2;
pub extern crate quote;
pub extern crate rustc_hash;
pub extern crate safe_index;
pub extern crate serde;
pub extern crate strum;
pub extern crate syn;

#[macro_use]
pub mod prelude;

#[macro_use]
mod mk_new_def;

mod color;
mod constant;
mod convert_case;
mod error;
mod hash_map;
mod label;
mod location;
mod scope;
mod r#type;

pub use syn::parse_quote;

pub mod conf;
pub mod keyword;
pub mod operator;
pub mod serialize;
pub mod synced;

pub mod graph {
    pub use super::{
        new_graph,
        petgraph::{graphmap::DiGraphMap, Direction},
        {color::Color, label::Label},
    };

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
}
