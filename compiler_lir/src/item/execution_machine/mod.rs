use self::{flows_context::FlowsContext, service_loop::ServiceLoop};

pub mod flows_context;
pub mod service_loop;

/// A execution-machine structure.
#[derive(Debug, PartialEq, Default)]
pub struct ExecutionMachine {
    /// The signals context from where components will get their inputs.
    pub flows_context: FlowsContext,
    /// The services loops.
    pub services_loops: Vec<ServiceLoop>,
}
