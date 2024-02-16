use std::collections::HashMap;

use crate::ast::node::Node;
use crate::error::{Error, TerminationError};
use crate::hir::{node::Node as HIRNode, once_cell::OnceCell};
use crate::symbol_table::{SymbolKind, SymbolTable};

use super::HIRFromAST;

impl HIRFromAST for Node {
    type HIR = HIRNode;

    fn hir_from_ast(
        self,
        symbol_table: &mut SymbolTable,
        errors: &mut Vec<Error>,
    ) -> Result<Self::HIR, TerminationError> {
        let Node {
            id,
            is_component,
            inputs,
            equations,
            contract,
            location,
        } = self;

        let id = symbol_table.get_node_id(&id, false, location.clone(), errors)?;
        let node_symbol = symbol_table
            .get_symbol(&id)
            .expect("there should be a symbol")
            .clone();
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
                        Ok((id, equation.hir_from_ast(symbol_table, errors)?))
                    })
                    .collect::<Vec<Result<_, _>>>()
                    .into_iter()
                    .collect::<Result<HashMap<_, _>, _>>()?;
                let contract = contract.hir_from_ast(symbol_table, errors)?;

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
}
