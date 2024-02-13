use std::collections::HashMap;

use crate::ast::node::Node;
use crate::error::{Error, TerminationError};
use crate::frontend::hir_from_ast::{
    contract::hir_from_ast as contract_hir_from_ast,
    equation::hir_from_ast as equation_hir_from_ast,
};
use crate::hir::{node::Node as HIRNode, once_cell::OnceCell};
use crate::symbol_table::{SymbolKind, SymbolTable};

/// Transform AST nodes into HIR nodes.
pub fn hir_from_ast(
    node: Node,
    symbol_table: &mut SymbolTable,
    errors: &mut Vec<Error>,
) -> Result<HIRNode, TerminationError> {
    let Node {
        id,
        is_component,
        inputs,
        equations,
        contract,
        location,
    } = node;

    let id = symbol_table.get_node_id(&id, false, location, errors)?;
    let node_symbol = symbol_table
        .get_symbol(&id)
        .expect("there should be a symbol");
    match node_symbol.kind() {
        SymbolKind::Node {
            inputs,
            outputs,
            locals,
            ..
        } => {
            // create local context with all signals
            symbol_table.local();
            symbol_table.restore_context(inputs.iter());
            symbol_table.restore_context(outputs.values());
            symbol_table.restore_context(locals.values());

            let unscheduled_equations = equations
                .into_iter()
                .map(|(signal, equation)| {
                    let id = symbol_table.get_signal_id(
                        &signal,
                        true,
                        equation.location.clone(),
                        errors,
                    )?;
                    Ok((id, equation_hir_from_ast(equation, symbol_table, errors)?))
                })
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<HashMap<_, _>, _>>()?;
            let contract = contract_hir_from_ast(contract, symbol_table, errors)?;

            symbol_table.global();

            Ok(HIRNode {
                id,
                unscheduled_equations,
                unitary_nodes: HashMap::new(),
                contract,
                location,
                graph: OnceCell::new(),
            })
        }
        _ => unreachable!(),
    }
}
