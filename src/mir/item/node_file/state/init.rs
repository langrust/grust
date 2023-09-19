use crate::common::constant::Constant;

/// A init function.
pub struct Init {
    /// The node's name.
    pub node_name: String,
    /// The initialization of the node's state.
    pub state_elements_init: Vec<StateElementInit>,
}

/// A state element structure for the initialization.
pub enum StateElementInit {
    /// A buffer initialization.
    BufferInit {
        /// The name of the buffer.
        identifier: String,
        /// The initial value.
        initial_value: Constant,
    },
    /// A called node initialization.
    CalledNodeInit {
        /// The name of the memory storage.
        identifier: String,
        /// The name of the called node.
        node_name: String,
    },
}
