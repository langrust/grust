use crate::lir::item::execution_machine::ExecutionMachine;
use std::collections::BTreeSet;

use flow_run_loop::rust_ast_from_lir as flow_run_loop_rust_ast_from_lir;

pub mod flow_run_loop;
pub mod instruction_flow;

/// Transform LIR execution-machine into items.
pub fn rust_ast_from_lir(
    execution_machine: ExecutionMachine,
    crates: &mut BTreeSet<String>,
) -> Vec<syn::Item> {
    flow_run_loop_rust_ast_from_lir(execution_machine.run_loop, crates)
}
