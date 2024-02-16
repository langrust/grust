use itertools::Itertools;

use crate::{
    hir::memory::{Buffer, CalledNode, Memory},
    lir::item::node_file::state::{init::StateElementInit, step::StateElementStep, StateElement},
    symbol_table::SymbolTable,
};

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
            .sorted_by_key(|(id, _)| id.clone()) // TODO why is it sorted?
            .for_each(
                |(
                    id,
                    Buffer {
                        typing,
                        initial_value,
                        expression,
                    },
                )| {
                    let name = symbol_table.get_name(&id);
                    elements.push(StateElement::Buffer {
                        identifier: name.clone(),
                        r#type: typing,
                    });
                    inits.push(StateElementInit::BufferInit {
                        identifier: name.clone(),
                        initial_value,
                    });
                    steps.push(StateElementStep {
                        identifier: name.clone(),
                        expression: expression.lir_from_hir(symbol_table),
                    });
                },
            );
        called_nodes
            .into_iter()
            .sorted_by_key(|(id, _)| id.clone()) // TODO why is it sorted?
            .for_each(|(id, CalledNode { node_id, signal_id })| {
                let name = symbol_table.get_name(&id);
                elements.push(StateElement::CalledNode {
                    identifier: name.clone(),
                    node_name: format!("{node_id}_{signal_id}"),
                });
                inits.push(StateElementInit::CalledNodeInit {
                    identifier: name.clone(),
                    node_name: format!("{node_id}_{signal_id}"),
                });
                // Because step function update state in place,
                // we don't need to update called nodes' state
                // steps.push(StateElementStep {
                //     identifier: id.clone(),
                //     expression: LIRExpression::Identifier { identifier: id },
                // });
            });

        (elements, inits, steps)
    }
}
