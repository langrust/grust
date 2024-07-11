prelude! {
    item::execution_machine::{
        service_handler::ServiceHandler,
        runtime_loop::RuntimeLoop,
    }
}

pub mod flows_context;
pub mod runtime_loop;
pub mod service_handler;

/// A execution-machine structure.
#[derive(Debug, PartialEq, Default)]
pub struct ExecutionMachine {
    /// The input flows.
    pub input_flows: Vec<InterfaceFlow>,
    /// The output flows.
    pub output_flows: Vec<InterfaceFlow>,
    /// The timing events.
    pub timing_events: Vec<TimingEvent>,
    /// The runtime loop.
    pub runtime_loop: RuntimeLoop,
    /// The services handlers.
    pub services_handlers: Vec<ServiceHandler>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceFlow {
    /// Path of the flow.
    pub path: syn::Path,
    /// The name of the flow.
    pub identifier: String,
    /// The type of the flow.
    pub r#type: Typ,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArrivingFlow {
    Channel(String, Typ, syn::Path),
    Period(String),
    Deadline(String),
}

/// A timing event structure.
#[derive(Clone, Debug, PartialEq)]
pub struct TimingEvent {
    /// The name of the timing event.
    pub identifier: String,
    /// Kind of timing event.
    pub kind: TimingEventKind,
}
#[derive(Clone, Debug, PartialEq)]
pub enum TimingEventKind {
    Period(u64),
    Timeout(u64),
}
