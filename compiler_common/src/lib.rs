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

mod constant;
mod convert_case;
mod error;
mod hash_map;
mod location;
mod scope;
mod r#type;

pub use syn::parse_quote;

pub mod conf;
pub mod graph;
pub mod keyword;
pub mod operator;
pub mod serialize;
pub mod synced;
