use crate::{
    common::r#type::Type,
    lir::item::state_machine::state::{init::Init, step::Step},
};

/// LIR [Init](crate::lir::item::node_file::state::init::Init) module.
pub mod init;
/// LIR [Step](crate::lir::item::node_file::state::step::Step) module.
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
