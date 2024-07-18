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
pub mod equiv;
pub mod keyword;
pub mod operator;
pub mod serialize;

pub mod graph {
    pub use super::{
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
}
