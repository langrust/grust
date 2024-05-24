use self::{flow_run_loop::FlowRunLoop, signals_context::SignalsContext};

pub mod flow_run_loop;
pub mod signals_context;

/// A execution-machine structure.
#[derive(Debug, PartialEq)]
pub struct ExecutionMachine {
    /// The signals context from where components will get their inputs.
    pub signals_context: SignalsContext,
    /// The run loop.
    pub run_loop: FlowRunLoop,
}
