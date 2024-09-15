use itertools::Itertools;

prelude! {
    hir::memory::{Buffer, CalledNode, Memory},
    lir::item::state_machine::state::{init::StateElementInit, step::StateElementStep, StateElement},
}

use super::LIRFromHIR;

impl Memory {
    /// Get state elements from memory.
    pub fn get_state_elements(
        self,
        symbol_table: &SymbolTable,
    ) -> (
        Vec<StateElement>,
        Vec<StateElementInit>,
        Vec<StateElementStep>,
    ) {
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
                        ..
                    },
                )| {
                    let mem_ident = format!("last_{}", ident);
                    elements.push(StateElement::Buffer {
                        identifier: mem_ident.clone(),
                        r#type: typing,
                    });
                    inits.push(StateElementInit::BufferInit {
                        identifier: mem_ident.clone(),
                        initial_expression: initial_expression.lir_from_hir(symbol_table),
                    });
                    steps.push(StateElementStep {
                        identifier: mem_ident.clone(),
                        expression: lir::Expr::ident(ident),
                    });
                },
            );
        called_nodes
            .into_iter()
            .sorted_by_key(|(id, _)| *id)
            .for_each(|(memory_id, CalledNode { node_id, .. })| {
                let memory_name = symbol_table.get_name(memory_id);
                let node_name = symbol_table.get_name(node_id);
                elements.push(StateElement::CalledNode {
                    identifier: memory_name.clone(),
                    node_name: node_name.clone(),
                });
                inits.push(StateElementInit::CalledNodeInit {
                    identifier: memory_name.clone(),
                    node_name: node_name.clone(),
                });
            });

        (elements, inits, steps)
    }
}
