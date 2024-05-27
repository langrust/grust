use self::{service_loop::ServiceLoop, signals_context::SignalsContext};

pub mod service_loop;
pub mod signals_context;

/// A execution-machine structure.
#[derive(Debug, PartialEq, Default)]
pub struct ExecutionMachine {
    /// The signals context from where components will get their inputs.
    pub signals_context: SignalsContext,
    /// The services loops.
    pub services_loops: Vec<ServiceLoop>,
}
