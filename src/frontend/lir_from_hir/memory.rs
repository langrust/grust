use itertools::Itertools;

use crate::{
    hir::memory::{Buffer, CalledNode, Memory},
    lir::item::{
        import::Import,
        node_file::state::{init::StateElementInit, step::StateElementStep, StateElement},
    },
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
            .sorted_by_key(|(id, _)| *id)
            .for_each(
                |(
                    memory_id,
                    Buffer {
                        typing,
                        initial_expression,
                        expression,
                    },
                )| {
                    let memory_name = symbol_table.get_name(memory_id);
                    elements.push(StateElement::Buffer {
                        identifier: memory_name.clone(),
                        r#type: typing,
                    });
                    inits.push(StateElementInit::BufferInit {
                        identifier: memory_name.clone(),
                        initial_expression: initial_expression.lir_from_hir(symbol_table),
                    });
                    steps.push(StateElementStep {
                        identifier: memory_name.clone(),
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

    /// Get imports from memory.
    pub fn get_imports(&self, symbol_table: &SymbolTable) -> Vec<Import> {
        let mut imports = self
            .buffers
            .values()
            .flat_map(
                |Buffer {
                     expression, typing, ..
                 }| {
                    let mut imports = expression.get_imports(symbol_table);
                    let mut typing_imports = typing.get_imports(symbol_table);
                    imports.append(&mut typing_imports);
                    imports
                },
            )
            .unique()
            .collect::<Vec<_>>();
        let mut called_node_imports = self
            .called_nodes
            .values()
            .flat_map(|CalledNode { node_id, .. }| {
                vec![Import::NodeFile(symbol_table.get_name(*node_id).clone())]
            })
            .unique()
            .collect::<Vec<_>>();

        imports.append(&mut called_node_imports);
        imports
    }
}
