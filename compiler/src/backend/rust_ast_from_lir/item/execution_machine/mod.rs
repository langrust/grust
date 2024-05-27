use crate::lir::item::execution_machine::ExecutionMachine;

use service_loop::rust_ast_from_lir as service_loop_rust_ast_from_lir;
use signals_context::rust_ast_from_lir as signals_context_rust_ast_from_lir;

pub mod flow_expression;
pub mod instruction_flow;
pub mod service_loop;
pub mod signals_context;

/// Transform LIR execution-machine into items.
pub fn rust_ast_from_lir(execution_machine: ExecutionMachine) -> Vec<syn::Item> {
    let mut items = signals_context_rust_ast_from_lir(execution_machine.signals_context);
    let mut other_items = execution_machine
        .services_loops
        .into_iter()
        .flat_map(service_loop_rust_ast_from_lir)
        .collect();
    items.append(&mut other_items);
    items
}
