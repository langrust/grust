//! LIR [StateMachine] module.

prelude! { just item::state_machine::{input::Input, state::State} }

pub mod input;
pub mod state;

/// A state-machine structure.
#[derive(Debug, PartialEq)]
pub struct StateMachine {
    /// The node's name.
    pub name: String,
    /// The input structure.
    pub input: Input,
    /// The state structure.
    pub state: State,
}
