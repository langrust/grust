//! LIR [State] module.

prelude! {
    item::state_machine::state::{init::Init, step::Step},
}

pub mod init;
pub mod step;

/// A node state structure.
#[derive(Debug, PartialEq)]
pub struct State {
    /// The node's name.
    pub node_name: String,
    /// The state's elements.
    pub elements: Vec<StateElement>,
    /// The step function.
    pub step: Step,
    /// The init function.
    pub init: Init,
}

/// A state element structure.
#[derive(Debug, PartialEq)]
pub enum StateElement {
    /// A buffer.
    Buffer {
        /// The name of the buffer.
        identifier: String,
        /// The type of the buffer.
        typ: Typ,
    },
    /// A called node memory.
    CalledNode {
        /// The name of the memory storage.
        identifier: String,
        /// The name of the called node.
        node_name: String,
    },
}
