prelude! { just
    item::{ Import, state_machine::{event::Event, input::Input, state::State} },
}

/// LIR [Event](crate::lir::item::state_machine::event::Event) module.
pub mod event;
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
    /// The event structure.
    pub event: Option<Event>,
    /// The state structure.
    pub state: State,
}
