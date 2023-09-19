use crate::mir::item::node_file::{import::Import, input::Input, state::State};

/// MIR [Import](crate::mir::item::node_file::import::Import) module.
pub mod import;
/// MIR [Input](crate::mir::item::node_file::input::Input) module.
pub mod input;
/// MIR [State](crate::mir::item::node_file::state::State) module.
pub mod state;

/// A node-file structure.
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
