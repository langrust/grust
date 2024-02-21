use itertools::Itertools;

use crate::{
    hir::unitary_node::UnitaryNode,
    lir::{
        expression::Expression as LIRExpression,
        item::node_file::{
            input::{Input, InputElement},
            state::{init::Init, step::Step, State},
            NodeFile,
        },
    },
    symbol_table::SymbolTable,
};

use super::LIRFromHIR;

impl LIRFromHIR for UnitaryNode {
    type LIR = NodeFile;

    fn lir_from_hir(self, symbol_table: &SymbolTable) -> Self::LIR {
        let UnitaryNode {
            id,
            statements,
            memory,
            ..
        } = self;

        // get node name
        let name = symbol_table.get_name(&id);

        // get node inputs
        let inputs = symbol_table
            .get_unitary_node_inputs(&id)
            .into_iter()
            .map(|id| {
                (
                    symbol_table.get_name(id).clone(),
                    symbol_table.get_type(id).clone(),
                )
            })
            .collect::<Vec<_>>();

        // get node output type
        let output_type = symbol_table.get_unitary_node_output_type(&id).clone();

        let output_expression = LIRExpression::Identifier {
            identifier: symbol_table.get_unitary_node_output_name(&id).clone(),
        };

        // collect imports from statements, inputs and output types and memory
        let mut imports = statements
            .iter()
            .flat_map(|equation| equation.get_imports(symbol_table))
            .unique()
            .collect::<Vec<_>>();
        let mut inputs_type_imports = inputs
            .iter()
            .flat_map(|(_, typing)| typing.get_imports(symbol_table))
            .unique()
            .collect::<Vec<_>>();
        let mut output_type_imports = output_type.get_imports(symbol_table);
        let mut memory_imports = memory.get_imports(symbol_table);

        // combining all imports and eliminate duplicates
        imports.append(&mut inputs_type_imports);
        imports.append(&mut output_type_imports);
        imports.append(&mut memory_imports);
        let imports = imports.into_iter().unique().collect::<Vec<_>>();

        let (elements, state_elements_init, state_elements_step) =
            memory.get_state_elements(symbol_table);

        NodeFile {
            name: name.clone(),
            imports,
            input: Input {
                node_name: name.clone(),
                elements: inputs
                    .into_iter()
                    .map(|(identifier, r#type)| InputElement { identifier, r#type })
                    .collect(),
            },
            state: State {
                node_name: name.clone(),
                elements,
                step: Step {
                    contract: self.contract.lir_from_hir(symbol_table),
                    node_name: name.clone(),
                    output_type,
                    body: statements
                        .into_iter()
                        .map(|equation| equation.lir_from_hir(symbol_table))
                        .collect(),
                    state_elements_step,
                    output_expression,
                },
                init: Init {
                    node_name: name.clone(),
                    state_elements_init,
                    invariant_initialisation: vec![], // TODO
                },
            },
        }
    }
}
