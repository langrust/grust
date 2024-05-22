use crate::lir::item::{
    import::Import,
    state_machine::{input::Input, state::State},
};

/// LIR [Input](crate::lir::item::state_machine::input::Input) module.
pub mod input;
/// LIR [State](crate::lir::item::state_machine::state::State) module.
pub mod state;

/// A state-machine structure.
#[derive(Debug, PartialEq)]
pub struct StateMachine {
    /// The node's name.
    pub name: String,
    /// The imports (used typedefs, functions and nodes).
    pub imports: Vec<Import>,
    /// The input structure.
    pub input: Input,
    /// The state structure.
    pub state: State,
}
