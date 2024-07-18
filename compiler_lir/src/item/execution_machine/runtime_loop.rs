prelude! { just item::execution_machine::ArrivingFlow }

/// The runtime loop structure.
#[derive(Debug, PartialEq, Default)]
pub struct RuntimeLoop {
    /// The input flow handlers.
    pub input_handlers: Vec<InputHandler>,
}

/// A flow structure.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InputHandler {
    /// Arriving flow.
    pub arriving_flow: ArrivingFlow,
    /// Delivered services.
    pub services: Vec<String>,
}
