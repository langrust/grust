use self::flow_run_loop::FlowRunLoop;

pub mod flow_run_loop;

/// A execution-machine structure.
#[derive(Debug, PartialEq)]
pub struct ExecutionMachine {
    /// The node's name.
    pub name: String,
    /// The run loop.
    pub run_loop: FlowRunLoop,
}
