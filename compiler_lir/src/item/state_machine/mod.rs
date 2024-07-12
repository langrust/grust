prelude! { just item::state_machine::{input::Input, state::State} }

/// LIR [Input](crate::lir::item::state_machine::input::Input) module.
pub mod input;
/// LIR [State](crate::lir::item::state_machine::state::State) module.
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
