/// MIR [Init](crate::mir::item::node_file::state::init::Init) module.
pub mod init;

/// A node state structure.
pub struct State {
    /// The node's name.
    pub node_name: String,
    /// The state's elements.
    pub elements: Vec<StateElement>,
}

/// A state element structure.
pub enum StateElement {
    /// A buffer.
    Buffer {
        /// The name of the buffer.
        identifier: String,
        /// The type of the buffer.
        r#type: Type,
    },
    /// A called node memory.
    CalledNode {
        /// The name of the memory storage.
        identifier: String,
        /// The name of the called node.
        node_name: String,
    },
}
