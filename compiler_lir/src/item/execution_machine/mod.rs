prelude! {
    just item::execution_machine::service_loop::ServiceLoop
}

pub mod flows_context;
pub mod service_loop;

/// A execution-machine structure.
#[derive(Debug, PartialEq, Default)]
pub struct ExecutionMachine {
    /// The services loops.
    pub services_loops: Vec<ServiceLoop>,
}
