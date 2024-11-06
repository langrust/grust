use itertools::Itertools;

prelude! {
    hir::memory::{Buffer, CalledNode, Memory},
    lir::state_machine::{StateElmInit, StateElmInfo, StateElmStep},
}

use super::LIRFromHIR;

impl Memory {
    /// Get state elements from memory.
    pub fn get_state_elements(
        self,
        symbol_table: &SymbolTable,
    ) -> (Vec<StateElmInfo>, Vec<StateElmInit>, Vec<StateElmStep>) {
        let Memory {
            buffers,
            called_nodes,
        } = self;

        let (mut elements, mut inits, mut steps) = (vec![], vec![], vec![]);
        buffers
            .into_iter()
            .sorted_by_key(|(id, _)| id.clone())
            .for_each(
                |(
                    _,
                    Buffer {
                        identifier: ident,
                        typing,
                        initial_expression,
                        id,
                        ..
                    },
                )| {
                    let scope = symbol_table.get_scope(id);
                    let mem_ident = format!("last_{}", ident);
                    elements.push(StateElmInfo::buffer(&mem_ident, typing));
                    inits.push(StateElmInit::buffer(
                        &mem_ident,
                        initial_expression.lir_from_hir(symbol_table),
                    ));
                    steps.push(StateElmStep::new(
                        mem_ident,
                        match scope {
                            Scope::Input => lir::Expr::input_access(ident),
                            Scope::Output | Scope::Local => lir::Expr::ident(ident),
                            Scope::VeryLocal => unreachable!(),
                        },
                    ));
                },
            );
        called_nodes
            .into_iter()
            .sorted_by_key(|(id, _)| *id)
            .for_each(|(memory_id, CalledNode { node_id, .. })| {
                let memory_name = symbol_table.get_name(memory_id);
                let node_name = symbol_table.get_name(node_id);
                elements.push(StateElmInfo::called_node(memory_name, node_name));
                inits.push(StateElmInit::called_node(memory_name, node_name));
            });

        (elements, inits, steps)
    }
}
