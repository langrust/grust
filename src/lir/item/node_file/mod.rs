use crate::lir::item::node_file::{import::Import, input::Input, state::State};

/// LIR [Import](crate::lir::item::node_file::import::Import) module.
pub mod import;
/// LIR [Input](crate::lir::item::node_file::input::Input) module.
pub mod input;
/// LIR [State](crate::lir::item::node_file::state::State) module.
pub mod state;

/// A node-file structure.
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct NodeFile {
    /// The node's name.
    pub name: String,
    /// The imports (called functions and nodes).
    pub imports: Vec<Import>,
    /// The input structure.
    pub input: Input,
    /// The state structure.
    pub state: State,
}
