use crate::lir::item::execution_machine::ExecutionMachine;
use std::collections::BTreeSet;

use service_loop::rust_ast_from_lir as service_loop_rust_ast_from_lir;

pub mod instruction_flow;
pub mod service_loop;

/// Transform LIR execution-machine into items.
pub fn rust_ast_from_lir(
    execution_machine: ExecutionMachine,
    crates: &mut BTreeSet<String>,
) -> Vec<syn::Item> {
    execution_machine
        .services_loops
        .into_iter()
        .flat_map(|service_loop| service_loop_rust_ast_from_lir(service_loop, crates))
        .collect()
}
