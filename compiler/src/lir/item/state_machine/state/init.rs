use crate::{ast::contract::Term, lir::expression::Expression};

/// A init function.
#[derive(Debug, PartialEq)]
pub struct Init {
    /// The node's name.
    pub node_name: String,
    /// The initialization of the node's state.
    pub state_elements_init: Vec<StateElementInit>,
    /// The invariant initialisation to prove.
    pub invariant_initialisation: Vec<Term>,
}

/// A state element structure for the initialization.
#[derive(Debug, PartialEq)]
pub enum StateElementInit {
    /// A buffer initialization.
    BufferInit {
        /// The name of the buffer.
        identifier: String,
        /// The initial value.
        initial_expression: Expression,
    },
    /// A called node initialization.
    CalledNodeInit {
        /// The name of the memory storage.
        identifier: String,
        /// The name of the called node.
        node_name: String,
    },
}
