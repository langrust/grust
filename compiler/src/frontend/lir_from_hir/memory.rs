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
        buffers.into_iter().sorted_by_key(|(id, _)| *id).for_each(
            |(
                memory_id,
                Buffer {
                    typing,
                    initial_expression,
                    expression,
                },
            )| {
                let memory_name = symbol_table.get_name(memory_id).clone();
                elements.push(StateElement::Buffer {
                    identifier: memory_name.clone(),
                    r#type: typing,
                });
                inits.push(StateElementInit::BufferInit {
                    identifier: memory_name.clone(),
                    initial_expression: initial_expression.lir_from_hir(symbol_table),
                });
                steps.push(StateElementStep {
                    identifier: memory_name,
                    expression: expression.lir_from_hir(symbol_table),
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
