use crate::lir::item::execution_machine::ExecutionMachine;

use flows_context::rust_ast_from_lir as flows_context_rust_ast_from_lir;
use service_loop::rust_ast_from_lir as service_loop_rust_ast_from_lir;

pub mod flow_expression;
pub mod flows_context;
pub mod instruction_flow;
pub mod service_loop;

/// Transform LIR execution-machine into items.
pub fn rust_ast_from_lir(execution_machine: ExecutionMachine) -> Vec<syn::Item> {
    let mut items = flows_context_rust_ast_from_lir(execution_machine.flows_context);
    let mut other_items = execution_machine
        .services_loops
        .into_iter()
        .flat_map(service_loop_rust_ast_from_lir)
        .collect();
    items.append(&mut other_items);
    items
}
